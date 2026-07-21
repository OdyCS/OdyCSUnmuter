# Third-party notices

## `source2-demo`

This project uses [`source2-demo`](https://github.com/Rupas1k/source2-demo), a
Source 2 replay parser by Rupas1k and contributors.

The dependency is dual-licensed under the MIT License or Apache License 2.0.
This project uses it under the MIT option.

The setup script downloads version `0.5.8`, creates a local vendored copy, and
modifies the initial `CDemoFileInfo` lookup so that unavailable or incomplete
footer metadata can fall back to empty summary metadata. The global Cargo
registry copy is not modified.

Compiled release packages include the dependency's original `LICENSE-MIT`
file when it is available in the vendored source.

## Development attribution

Project maintained by Ody. Initial implementation, debugging, and
documentation were developed with assistance from OpenAI's ChatGPT.
