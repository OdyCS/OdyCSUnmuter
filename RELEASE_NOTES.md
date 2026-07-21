# OdyCSUnmuter Beta v0.1.0-beta.1

First public Windows x64 beta under the OdyCSUnmuter name.

## Confirmed behavior

Successfully restored globally muted player voice in two tested FACEIT demos.

## Installation

1. Download the attached Windows ZIP.
2. Extract all files.
3. Drag an untouched `.dem` onto `Run-OdyCSUnmuter.bat`.
4. Load the generated `*_OdyCSunmuted.dem`.

The compiled release does not require Rust.

## Recommended CS2 commands

```cfg
tv_listen_voice_indices -1
tv_listen_voice_indices_h -1
```

`cl_sanitize_muted_players 0` is optional and affects names/avatars only.

## Important

- Keep the original demo.
- The target must have recorded voice packets.
- All Steam identities are remapped in the output.
- The output may reveal voice beyond original listener masks.
- Verify the attached SHA-256 file.
