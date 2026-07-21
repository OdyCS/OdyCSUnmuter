# OdyCSUnmuter Beta

A Windows tool that rewrites downloaded Counter-Strike 2 demos so globally
communication-muted players can be heard during **offline demo playback**.

The working method clears recorded mute state and remaps every real Steam
identity consistently across player-controller state, `userinfo`, and voice
packets.

> **Status:** experimental public beta. Successfully tested on multiple FACEIT
> demos after simpler flag-only methods failed.

## Download for ordinary Windows users

Open **Releases** and download:

```text
OdyCSUnmuter-Beta-v0.1.0-beta.1-windows-x64.zip
```

The compiled release does **not** require Rust, Cargo, Visual Studio, or the
source code.

Do not download GitHub's automatic **Source code** ZIP unless you intend to
build the project yourself.

## Everything an ordinary user needs

- Windows 10 or later, x64
- Counter-Strike 2 installed
- A downloaded `.dem` file
- The compiled OdyCSUnmuter ZIP
- The CS2 developer console enabled
- Free disk space for another demo of roughly similar size

The demo must contain recorded voice packets. The tool cannot recreate audio
that was never stored.

## How to use

1. Extract the entire release ZIP.
2. Keep an untouched copy of the original `.dem`.
3. Drag the original demo onto `Run-OdyCSUnmuter.bat`.
4. Wait for all four stages.
5. The program creates:

```text
originalname_OdyCSunmuted.dem
```

6. Fully close and reopen CS2 before the first test.

7. Load the generated *_OdyCSunmuted.dem normally.

For GOTV demos, make sure all player voice slots are enabled:

tv_listen_voice_indices -1
tv_listen_voice_indices_h -1

Or alternatively use 3rd party programs or manually select and convert to binary the desired player you want to be heard and replace the -1 with that number

### Is `cl_sanitize_muted_players 0` required?

No. It affects hidden names and avatars, not voice audio.

It is optional:

```cfg
cl_sanitize_muted_players 0
```

## Confirm that the target's audio exists

Read the **VOICE AUDIO SPEAKER REPORT** after processing.

The target should appear by name with non-zero `Audio bytes`. If the player is
absent or has zero bytes, their speech was not recorded in that demo.

## What the tool changes

### Mute/listener state

```text
CCSPlayerController.m_bHasCommunicationAbuseMute -> false
CCSPlayerController.m_uiCommunicationMuteFlags   -> 0
CUserMessageVoiceMask.ban_masks                  -> 0
CUserMessageVoiceMask.mod_enable                 -> true
CSvcMsgVoiceData.audible_mask                    -> all listeners
```

Existing game-rules listener masks are expanded.

### Steam identity

Every non-zero real Steam identity receives a unique synthetic SteamID64.
The same replacement is written to:

```text
CCSPlayerController.m_steamID
userinfo CMsgPlayerInfo.xuid
userinfo CMsgPlayerInfo.steamid
CSvcMsgVoiceData.xuid
```

Names and recorded voice bytes are preserved.

## Important consequences

- Every player's Steam ID is remapped in the output.
- Avatars/profile links may no longer match.
- The output may expose voice beyond the original listener masks.
- The rewritten demo still contains names and voice audio.
- Do not publish private communications without permission.
- This modifies only the output demo, not accounts or penalties.
- Future CS2 updates may break compatibility.

## Build from source

### Requirements

- Windows 10 or later, x64
- Stable Rust and Cargo installed through `rustup`
- Internet access during the first build
- Microsoft C++ Build Tools may be required

Installing Rust with `rustup` also installs Cargo.

### Steps

1. Clone or download the repository.
2. Drag a demo onto `Build-and-Run-OdyCSUnmuter.bat`.

For unusually deep demos:

```bat
set ODYCSUNMUTER_STACK_MB=256
Build-and-Run-OdyCSUnmuter.bat "C:\path\match.dem"
```

## Credits

Project created, developed, and maintained by Ody.

Built using [`source2-demo`](https://github.com/Rupas1k/source2-demo) by
Rupas1k and contributors under its MIT licensing option.

## License

MIT License — `Copyright (c) 2026 Ody`.

See [LICENSE](LICENSE) and [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

## Disclaimer

Not affiliated with Valve, Steam, FACEIT, Counter-Strike, or the
`source2-demo` authors.
