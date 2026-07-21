OdyCSUnmuter Beta v0.1.0-beta.1 — Windows x64
============================================================

CONFIRMED STATUS
----------------
Experimental public beta. Successfully tested on mulitple FACEIT demos.

WHAT AN ORDINARY USER NEEDS
---------------------------
- Windows 10 or later, x64
- Counter-Strike 2 installed
- A downloaded CS2 .dem file
- The compiled OdyCSUnmuter release ZIP
- The CS2 developer console enabled
- Free disk space for another demo file of roughly similar size

RUST IS NOT REQUIRED FOR THE COMPILED RELEASE.

Rust, Cargo, internet access, and possibly Microsoft C++ Build Tools are
required ONLY when building from source.

USE
---
1. Extract the ENTIRE release ZIP.
2. Keep an untouched backup of the original .dem.
3. Drag the original demo onto Run-OdyCSUnmuter.bat.
4. Wait until all four stages finish.
5. The program creates:

       originalname_OdyCSunmuted.dem

6. Fully close and reopen CS2 before the first test.
7. Run these CS2 console commands (adjust the number if you want to hear specific players instead of everyone):

       tv_listen_voice_indices -1
       tv_listen_voice_indices_h -1

8. Load the new *_OdyCSunmuted.dem file.

OPTIONAL NAME/AVATAR COMMAND
----------------------------
       cl_sanitize_muted_players 0

This affects hidden names and avatars only. It is not the audio fix.

IMPORTANT LIMITATIONS
---------------------
- The demo must contain the target player's recorded voice packets.
- The target must appear in the VOICE AUDIO SPEAKER REPORT with non-zero
  audio bytes.
- The tool cannot recreate speech that was never recorded.
- Every player's Steam identity is remapped in the output.
- Player names and recorded voice remain; avatars/profile links may differ.
- The output may reveal voice beyond the original listener restrictions.
- Keep rewritten demos private unless sharing is authorized.
- The original demo is never intentionally overwritten.
- This does not alter accounts, penalties, servers, or live games.
- Future CS2 demo-format changes may break compatibility.

WINDOWS SMARTSCREEN
-------------------
The executable is unsigned. Verify the GitHub release checksum or build from
source when stronger assurance is needed.

LICENSE
-------
MIT License — Copyright (c) 2026 Ody.

NOT AFFILIATED
--------------
Not affiliated with Valve, Steam, FACEIT, Counter-Strike, or the source2-demo
authors.
