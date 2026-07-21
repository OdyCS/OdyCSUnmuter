use anyhow::{bail, Context as AnyhowContext, Result};
use source2_demo::prelude::*;
use source2_demo::proto::{CMsgPlayerInfo, CSvcMsgVoiceData, CUserMessageVoiceMask};
use source2_demo::writer::*;
use std::collections::{BTreeMap, HashMap};
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::thread;

const VERSION: &str = "0.1.0-beta.1";
const DEFAULT_STACK_MB: usize = 128;

// SteamID64 base for an individual account, plus a high but valid account-id
// range. Each real player receives a unique synthetic SteamID64.
const STEAM_ID64_INDIVIDUAL_BASE: u64 = 76_561_197_960_265_728;
const SYNTHETIC_ACCOUNT_ID_BASE: u64 = 4_000_000_000;

#[derive(Default, Debug)]
struct SpeakerStats {
    packets: u64,
    packets_with_audio: u64,
    audio_bytes: u64,
    audible_mask_missing: u64,
    audible_mask_zero: u64,
    audible_mask_all: u64,
    audible_mask_other: u64,
}

#[derive(Default)]
struct ClearCommunicationMute {
    // Consistent identity remapping.
    id_map: HashMap<u64, u64>,
    next_synthetic_id: u64,
    names: HashMap<u64, String>,
    userinfo_rows_seen: u64,
    userinfo_rows_remapped: u64,
    player_controller_ids_seen: u64,
    player_controller_ids_remapped: u64,

    // Entity mute state.
    abuse_updates: u64,
    abuse_true_updates: u64,
    flag_updates: u64,
    flag_nonzero_updates: u64,

    // CUserMessageVoiceMask state.
    voice_mask_messages: u64,
    voice_mask_messages_changed: u64,
    gamerules_words_changed: u64,
    gamerules_bits_enabled: u64,
    ban_words_cleared: u64,
    ban_bits_cleared: u64,
    mod_enable_changes: u64,

    // CSvcMsgVoiceData state.
    voice_packets: u64,
    voice_packets_with_audio: u64,
    voice_audio_bytes: u64,
    audible_masks_changed: u64,
    voice_packet_ids_remapped: u64,
    speakers: BTreeMap<(u64, i32), SpeakerStats>,
}

impl ClearCommunicationMute {
    fn remap_id(&mut self, original: u64) -> u64 {
        if original == 0 {
            return 0;
        }

        if let Some(mapped) = self.id_map.get(&original) {
            return *mapped;
        }

        let mapped = STEAM_ID64_INDIVIDUAL_BASE
            + SYNTHETIC_ACCOUNT_ID_BASE
            + self.next_synthetic_id;

        self.next_synthetic_id += 1;
        self.id_map.insert(original, mapped);
        mapped
    }

    fn remember_name(&mut self, original: u64, name: Option<&str>) {
        if original == 0 {
            return;
        }

        if let Some(name) = name {
            if !name.is_empty() {
                self.names
                    .entry(original)
                    .or_insert_with(|| name.to_owned());
            }
        }
    }
}

#[rewriter]
impl ClearCommunicationMute {
    // The global communication-abuse entity state.
    #[rewrite_field(
        class = "CCSPlayerController",
        field = "m_bHasCommunicationAbuseMute"
    )]
    fn clear_abuse_mute(&mut self, value: bool) -> bool {
        self.abuse_updates += 1;
        if value {
            self.abuse_true_updates += 1;
        }
        false
    }

    // The accompanying entity communication flags.
    #[rewrite_field(
        class = "CCSPlayerController",
        field = "m_uiCommunicationMuteFlags"
    )]
    fn clear_mute_flags(&mut self, value: u32) -> u32 {
        self.flag_updates += 1;
        if value != 0 {
            self.flag_nonzero_updates += 1;
        }
        0
    }

    // Replace the real Steam identity stored on each player controller.
    #[rewrite_field(
        class = "CCSPlayerController",
        field = "m_steamID"
    )]
    fn remap_player_controller_steam_id(&mut self, value: u64) -> u64 {
        self.player_controller_ids_seen += 1;

        if value == 0 {
            return 0;
        }

        self.player_controller_ids_remapped += 1;
        self.remap_id(value)
    }

    // Replace the same identity in the userinfo table while preserving names.
    #[rewrite_string_table_entry]
    fn remap_userinfo(
        &mut self,
        table_name: &str,
        entry: &mut StringTableEntryUpdate,
    ) -> Result<(), ParserError> {
        if table_name != "userinfo" {
            return Ok(());
        }

        let Some(value) = entry.value_mut() else {
            return Ok(());
        };

        self.userinfo_rows_seen += 1;

        let mut player = CMsgPlayerInfo::decode(value.as_slice())?;
        let original = player
            .xuid
            .or(player.steamid)
            .unwrap_or(0);

        self.remember_name(original, player.name.as_deref());

        if original != 0 {
            let mapped = self.remap_id(original);
            player.xuid = Some(mapped);
            player.steamid = Some(mapped);
            self.userinfo_rows_remapped += 1;
            *value = player.encode_to_vec();
        }

        Ok(())
    }

    #[rewrite_packet_message]
    fn clear_voice_mask(
        &mut self,
        message: &mut CUserMessageVoiceMask,
    ) -> Result<MessageRewrite, ParserError> {
        self.voice_mask_messages += 1;
        let mut changed = false;

        for mask in &mut message.gamerules_masks {
            if *mask != u32::MAX {
                self.gamerules_words_changed += 1;
                self.gamerules_bits_enabled += u64::from((!*mask).count_ones());
                *mask = u32::MAX;
                changed = true;
            }
        }

        for mask in &mut message.ban_masks {
            if *mask != 0 {
                self.ban_words_cleared += 1;
                self.ban_bits_cleared += u64::from(mask.count_ones());
                *mask = 0;
                changed = true;
            }
        }

        if message.mod_enable != Some(true) {
            message.mod_enable = Some(true);
            self.mod_enable_changes += 1;
            changed = true;
        }

        if changed {
            self.voice_mask_messages_changed += 1;
        }

        Ok(MessageRewrite::Rewrite)
    }

    #[rewrite_packet_message]
    fn rewrite_voice_packet(
        &mut self,
        message: &mut CSvcMsgVoiceData,
    ) -> Result<MessageRewrite, ParserError> {
        self.voice_packets += 1;

        let original_xuid = message.xuid.unwrap_or(0);
        let client = message.client_deprecated.unwrap_or(-1);
        let audio_bytes = message
            .audio
            .as_ref()
            .map(|audio| audio.voice_data().len() as u64)
            .unwrap_or(0);

        if audio_bytes > 0 {
            self.voice_packets_with_audio += 1;
            self.voice_audio_bytes += audio_bytes;
        }

        let speaker = self
            .speakers
            .entry((original_xuid, client))
            .or_default();

        speaker.packets += 1;
        speaker.audio_bytes += audio_bytes;

        if audio_bytes > 0 {
            speaker.packets_with_audio += 1;
        }

        match message.audible_mask {
            None => speaker.audible_mask_missing += 1,
            Some(0) => speaker.audible_mask_zero += 1,
            Some(-1) => speaker.audible_mask_all += 1,
            Some(_) => speaker.audible_mask_other += 1,
        }

        if message.audible_mask != Some(-1) {
            message.audible_mask = Some(-1);
            self.audible_masks_changed += 1;
        }

        // Unlike V6, retain a valid, unique speaker identity. The same
        // synthetic ID is written to player controllers and userinfo.
        if original_xuid != 0 {
            message.xuid = Some(self.remap_id(original_xuid));
            self.voice_packet_ids_remapped += 1;
        }

        Ok(MessageRewrite::Rewrite)
    }
}

fn default_output(input: &Path) -> PathBuf {
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("demo");

    input.with_file_name(format!("{stem}_OdyCSunmuted.dem"))
}

fn temporary_output(output: &Path) -> PathBuf {
    let filename = output
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("demo_OdyCSunmuted.dem");

    output.with_file_name(format!("{filename}.partial"))
}

fn inspect_header(input: &Path) -> Result<()> {
    let mut file = File::open(input)
        .with_context(|| format!("Could not open input demo: {}", input.display()))?;

    let length = file
        .metadata()
        .with_context(|| format!("Could not read input size: {}", input.display()))?
        .len();

    let mut header = [0u8; 16];
    file.read_exact(&mut header)
        .context("Could not read the 16-byte demo header.")?;

    if &header[..8] != b"PBDEMS2\0" {
        bail!("This is not a PBDEMS2 Source 2 demo.");
    }

    let file_info =
        u32::from_le_bytes(header[8..12].try_into().expect("four bytes")) as u64;
    let spawn_groups =
        u32::from_le_bytes(header[12..16].try_into().expect("four bytes")) as u64;

    println!("Input size:           {length}");
    println!("File-info offset:     {file_info}");
    println!("Spawn-groups offset:  {spawn_groups}");

    if file_info >= length || spawn_groups >= length {
        println!(
            "Detected a missing/incomplete footer. The local parser patch will\n\
             use empty summary metadata and continue at byte 16."
        );
    } else {
        println!("Footer offsets are inside the file.");
    }

    Ok(())
}

fn parse_stack_size() -> usize {
    let mb = std::env::var("ODYCSUNMUTER_STACK_MB")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(DEFAULT_STACK_MB)
        .clamp(16, 1024);

    mb * 1024 * 1024
}

fn print_identity_report(stats: &ClearCommunicationMute) {
    println!();
    println!("IDENTITY REMAP REPORT");
    println!(
        "  Userinfo rows seen: {} (remapped: {})",
        stats.userinfo_rows_seen, stats.userinfo_rows_remapped
    );
    println!(
        "  Player-controller Steam-ID updates: {} (remapped: {})",
        stats.player_controller_ids_seen, stats.player_controller_ids_remapped
    );
    println!(
        "  Voice-packet XUIDs remapped: {}",
        stats.voice_packet_ids_remapped
    );
    println!("  Unique real identities remapped: {}", stats.id_map.len());
}

fn print_speaker_report(stats: &ClearCommunicationMute) {
    println!();
    println!("VOICE AUDIO SPEAKER REPORT");
    println!(
        "  Name                     Original XUID           Synthetic XUID          Client  Packets  Audio bytes"
    );
    println!(
        "  -----------------------  ----------------------  ----------------------  ------  -------  -----------"
    );

    let mut speakers: Vec<_> = stats.speakers.iter().collect();
    speakers.sort_by(|a, b| {
        b.1.audio_bytes
            .cmp(&a.1.audio_bytes)
            .then_with(|| b.1.packets.cmp(&a.1.packets))
            .then_with(|| a.0.cmp(b.0))
    });

    for ((xuid, client), speaker) in speakers {
        let name = stats
            .names
            .get(xuid)
            .map(String::as_str)
            .unwrap_or("<name unavailable>");

        let mapped = stats.id_map.get(xuid).copied().unwrap_or(0);

        println!(
            "  {:<23}  {:>20}  {:>20}  {:>6}  {:>7}  {:>11}",
            name,
            xuid,
            mapped,
            client,
            speaker.packets,
            speaker.audio_bytes,
        );
        println!(
            "    original masks: missing={} zero={} all={} other={}",
            speaker.audible_mask_missing,
            speaker.audible_mask_zero,
            speaker.audible_mask_all,
            speaker.audible_mask_other,
        );
    }

    if stats.speakers.is_empty() {
        println!("  No CSvcMsgVoiceData speaker entries were encountered.");
    }
}

fn real_main(args: Vec<OsString>) -> Result<()> {
    let mut args = args.into_iter();

    let input = args
        .next()
        .map(PathBuf::from)
        .context("Usage: OdyCSUnmuter <input.dem> [output.dem]")?;

    let output = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| default_output(&input));

    if input == output {
        bail!("Input and output paths must be different.");
    }

    let partial = temporary_output(&output);
    inspect_header(&input)?;

    if output.exists() {
        fs::remove_file(&output)
            .with_context(|| format!("Could not replace old output: {}", output.display()))?;
    }

    if partial.exists() {
        fs::remove_file(&partial)
            .with_context(|| format!("Could not remove old partial file: {}", partial.display()))?;
    }

    println!();
    println!("[1/4] Opening demo parser...");

    let reader = BufReader::new(
        File::open(&input)
            .with_context(|| format!("Could not open: {}", input.display()))?,
    );

    let writer = BufWriter::new(
        File::create(&partial)
            .with_context(|| format!("Could not create temporary output: {}", partial.display()))?,
    );

    let mut demo_writer = DemoWriter::from_reader(reader, writer)
        .context("Could not initialize the demo writer.")?;

    println!("[2/4] Registering mute and consistent identity rewrites...");
    let stats = demo_writer.register_rewriter::<ClearCommunicationMute>();

    println!("[3/4] Parsing and rewriting the demo...");
    if let Err(error) = demo_writer.run() {
        drop(demo_writer);
        let _ = fs::remove_file(&partial);

        return Err(error).context(
            "Demo rewriting failed. The incomplete temporary output was deleted.",
        );
    }

    println!("[4/4] Flushing output...");
    let (_parser, mut writer) = demo_writer.into_parts();

    writer.flush().context("Could not flush the rewritten demo.")?;
    writer
        .get_ref()
        .sync_all()
        .context("Could not sync the rewritten demo to disk.")?;

    drop(writer);

    fs::rename(&partial, &output).with_context(|| {
        format!(
            "Rewrite completed, but the temporary file could not be renamed from {} to {}",
            partial.display(),
            output.display()
        )
    })?;

    let stats = stats.borrow();

    println!();
    println!("Created: {}", output.display());

    println!();
    println!("ENTITY MUTE RESULTS");
    println!(
        "  m_bHasCommunicationAbuseMute updates: {} (true values cleared: {})",
        stats.abuse_updates, stats.abuse_true_updates
    );
    println!(
        "  m_uiCommunicationMuteFlags updates: {} (non-zero values cleared: {})",
        stats.flag_updates, stats.flag_nonzero_updates
    );

    println!();
    println!("VOICE-MASK RESULTS");
    println!(
        "  CUserMessageVoiceMask messages: {} (messages changed: {})",
        stats.voice_mask_messages, stats.voice_mask_messages_changed
    );
    println!(
        "  Game-rules mask words expanded: {} (bits enabled: {})",
        stats.gamerules_words_changed, stats.gamerules_bits_enabled
    );
    println!(
        "  Ban-mask words cleared: {} (bits cleared: {})",
        stats.ban_words_cleared, stats.ban_bits_cleared
    );
    println!("  mod_enable values forced on: {}", stats.mod_enable_changes);

    println!();
    println!("VOICE-PACKET RESULTS");
    println!("  CSvcMsgVoiceData packets: {}", stats.voice_packets);
    println!(
        "  Packets containing audio bytes: {}",
        stats.voice_packets_with_audio
    );
    println!("  Total encoded voice bytes: {}", stats.voice_audio_bytes);
    println!(
        "  Per-packet audible masks changed to all: {}",
        stats.audible_masks_changed
    );

    print_identity_report(&stats);
    print_speaker_report(&stats);

    println!();
    println!(
        "RESULT: The patcher preserved a valid per-player speaker identity while replacing\n\
         each real Steam ID consistently in userinfo, player-controller state,\n\
         and voice packets."
    );
    println!(
        "IMPORTANT: Confirm that the target player's NAME appears in the report\n\
         with non-zero audio bytes before testing the output in CS2."
    );

    Ok(())
}

fn launch() -> Result<()> {
    println!("OdyCSUnmuter Beta v{VERSION}");
    println!();

    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    let stack_size = parse_stack_size();

    println!(
        "Launching OdyCSUnmuter parser worker with {} MB stack...",
        stack_size / (1024 * 1024)
    );

    let worker = thread::Builder::new()
        .name("cs2-demo-rewrite".to_owned())
        .stack_size(stack_size)
        .spawn(move || real_main(args))
        .context("Could not create the large-stack parser thread.")?;

    match worker.join() {
        Ok(result) => result,
        Err(_) => bail!("The parser worker panicked. No completed output should be used."),
    }
}

fn main() {
    if let Err(error) = launch() {
        eprintln!();
        eprintln!("ERROR: {error:#}");
        std::process::exit(1);
    }
}
