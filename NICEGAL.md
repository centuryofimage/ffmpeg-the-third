# Nicegal integration delta

This branch starts from upstream commit
`47eb652418f56f8010e640946b105cd5614701f7`, published as
`ffmpeg-the-third` 6.0.0+ffmpeg-9.0 (WTFPL; see `LICENSE`). The matching crates.io
archive has SHA-256
`cf29dd4474510215c218b491acf6c06d27c42cf3579abdc20936290a3817ca31`.

The application uses the wrapper with a deliberately narrow native FFmpeg build.
Local manifest changes disable upstream unit, integration, documentation,
example, and benchmark targets during workspace checks. `src/lib.rs` also gates
the upstream test build because Cargo `--all-targets` can override the library
target's `test = false`. The `ffmpeg-sys-the-third` dependency uses crates.io,
as the published crate does, rather than the repository's bundled path.

No wrapper API or implementation has been changed in this integration commit.
