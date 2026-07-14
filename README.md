# Sonata

A cross-platform Rust engine for neural TTS models.

> **This is a fork of [mush42/sonata](https://github.com/mush42/sonata).**
> See [Fork changes](#fork-changes) for what differs from upstream.


## Supported models

* [Piper](https://github.com/rhasspy/piper)


# Fork changes

## Seamless streaming: overlap-add instead of fade-to-silence

Streaming synthesis produced audible pops and clicks at chunk boundaries — a
known issue with the upstream streaming path.

**Status:** verified numerically and builds clean; not yet validated by ear on
real speech.

**Cause.** `SpeechStreamer::process_chunk_audio` called `audio.crossfade(42)`,
but that method is not a crossfade at all: it fades each chunk to *silence* at
both ends and the chunks are then concatenated. Two fades to zero back to back
leave an amplitude dip at every seam — a gap, not a seam. Measured on a
continuous 220 Hz sine: RMS at the boundary dropped to **0.308** where an
undisturbed signal reads **0.707**. More than half the amplitude, gone, at every
chunk boundary.

**Fix.** The overlap was already there. `AdaptiveMelChunker` deliberately steps
back `chunk_padding * 2` frames at the head of each chunk, so consecutive chunks
describe *the same audio* across the seam. Those samples were being sliced away
and thrown out. They are now kept and overlap-added.

The blend uses **equal-gain (linear)** ramps, `fade_in + fade_out == 1`, not the
equal-power (sin/cos) curve normally reached for. Equal-power is correct for
mixing *uncorrelated* signals, where power adds but amplitude does not. Here both
halves of the seam are the same waveform, so amplitude adds directly and sin+cos
would peak at sqrt(2) — a +3 dB bump at every boundary. Linear gains sum to unity
and reconstruct the waveform exactly.

Verified against an unbroken sine: maximum deviation **0.00000006** (float noise).
Output length, peak sample-to-sample delta and RMS all match the reference.

Changed files:
- `crates/audio/ops/src/samples.rs` — adds `split_tail` and `overlap_add_head`
- `crates/sonata/models/piper/src/lib.rs` — streamer keeps the seam; chunker no
  longer discards the overlap (`start_padding = 0`); `HOP_LENGTH` named

## eSpeak-ng link path

`build.rs` failed to link `ucd` on Windows. Fixed in `0d91d64`.


# Crates

- `espeak-phonemizer`: Converts text to `IPA` phonemes using a patched version of eSpeak-ng
- `sonata-model`: Handles model loading and inference using `onnxruntime` via `ort`
- `sonata-synth`: Wraps `SonataModel` and adds synthesized speech post-processing, including changing prosody. Also provides different modes of parallelism.
- `sonata-grpc`: [GRPC](https://grpc.io/) frontend for sonata
- `libsonata`: C-API binding to sonata
- `sonata-python`: Python bindings to `sonata-synth` using `pyo3`
- `sonic-sys`: Rust FFI bindings to [Sonic](https://github.com/waywardgeek/sonic): a `C` library for controlling various aspects of generated speech, such as rate, volume, and pitch

# A note on testing

Some packages, such as `espeak-phonemizer`, include tests. Running `cargo test` from the root of the workspace will likely fail, because `cargo` does not load `config` from sub packages when ran from the workspace root.

On Windows you need to add `espeak-ng.dll` to the library search path by modifying the **PATH** environment variable.

For example, to add `espeak-ng.dll` to your path when building for the `x86_64-pc-windows-msvc` target, run the following command before `cargo test`:

```cmd
set PATH=%PATH%;{repo_path}\deps\windows\espeak-ng-build\i686\bin
```

Replace `repo_path` with the absolute path to the repository.

Then `cd` to the package, and run `cargo test` from there.

# License

Copyright (c) 2023 Musharraf Omer. This code is licensed under the  MIT license.

