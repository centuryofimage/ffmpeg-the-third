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

Safe API additions:

- `format::input_with_dictionary_and_interrupt` combines format options,
  per-stream probe options, and an owned callback.
- `Input::find_stream_info` copies probe options per existing stream and frees
  the dictionaries on success or failure.
- `ParametersRef::side_data` borrows coded side-data bytes on FFmpeg 7+.

`input_with_interrupt` now retains its callback through native close and releases
it on failed opening/probing. Callbacks require `Send + 'static` because inputs
can move across threads; invocation is serialized. Callback panics still abort.
The callback owner lives with the format context destructor, so moving or swapping
contexts does not separate the native pointer from its callback allocation.
Regression tests live in the application's `tests/ffmpeg_input.rs`, alongside
its decoder/rotation fixture tests, while the upstream test suite stays disabled.
