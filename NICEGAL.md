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
The local default feature set is `static`, `filter`, `format`, and
`software-scaling`, matching the application dependency. Cargo includes this path
dependency in workspace-wide commands on Windows, so the upstream defaults would
also request unbuilt `avdevice` and `swresample` libraries.

This branch (`nicegal/integration`) is upstream master plus the following wrapper changes,
each prepared as its own upstream branch (`upstream-pr/*`) except the last:

- `Input::find_stream_info(&Dictionary)` copies probe options per existing stream
  (`upstream-pr/find-stream-info`).
- `ParametersRef::side_data` borrows coded side-data bytes on FFmpeg 7+
  (`upstream-pr/side-data`).
- `input_with_interrupt` retains its callback through native close instead of leaking it, and
  callbacks require `Send + 'static` (`upstream-pr/interrupt-leak`).
- `format::input_with_dictionary_and_interrupt` combines format options, per-stream probe
  options, and an owned callback. This is local only: upstream's pending `format::context` rework
  would turn it into builder options, so it is not proposed upstream yet.

Regression tests live in the application's `tests/ffmpeg_input.rs`, alongside its
decoder/rotation fixture tests, while the upstream test suite stays disabled.
