# G&W render fix

**Stock Mr. Game & Watch. Less buffer readback. About 2.1× the measured Eden FPS.**

An experimental Rust Skyline plugin for Super Smash Bros. Ultimate. It narrows
one verified shader's writable buffer range, then restores the original binding.
The game keeps its stock geometry and gameplay.

[Download on Forgejo](https://git.syntaxbyexample.com/intpa/gnw-render-fix/releases/tag/v0.1.1-experimental)
· [Download on GitHub](https://github.com/crashbandicode/gnw-render-fix/releases/tag/v0.1.1-experimental)
· [Benchmark & settings](EDEN_BENCHMARK.md)
· [Technical validation](VALIDATION.md)

## Before → after → return control

Re-verified **October 9, 2026**, using the corrected **`v0.1.1-experimental`** NRO.
Four stock G&Ws idle on the Training stage, three CPUs set to Stop, one Eden
instance, and the same executable and effective settings in all three passes.

| 60-second window | Before — disabled | After — enabled | Return control — disabled |
| --- | ---: | ---: | ---: |
| **Median FPS** | **28.00** | **59.99** | **29.00** |
| Mean FPS | 27.38 | 59.03 | 29.05 |
| Sampled min–max FPS | 20.02–29.01 | 41.99–60.05 | 26.99–30.02 |
| Median GPU utilization | 79% | 28% | 81% |

| Before: plugin disabled | After: corrected plugin enabled |
| --- | --- |
| ![Four stock G&Ws before enabling the plugin](docs/evidence/2026-10-09/before.png) | ![Four stock G&Ws with the corrected plugin](docs/evidence/2026-10-09/after.png) |

Full, unretouched screenshots taken immediately before each sample window.
Extra CPU costume colors and idle animation phases are game-assigned and vary
between launches. [Return-control screenshot](docs/evidence/2026-10-09/control.png).

Each window contains 60 one-second CLI FPS observations after an additional
30-second warm-up; every observation reported zero shaders compiling. This is
about **2.07–2.14× median FPS** relative to the two disabled runs, not a claim of
perfectly locked 60 FPS. No FPS mod, replacement skin, Training Modpack, Deluxe,
or unsafe Eden readback bypass was loaded.

**Test machine:** Surface Laptop Studio · Intel `i7-11370H` (4 cores / 8 threads)
· 32 GiB RAM · RTX `3050 Ti Laptop` (4 GiB) · NVIDIA `610.57.04` · Ubuntu `26.04.1`.
**Effective settings:** Vulkan, High/“ACCURATE” GPU accuracy, Docked `1×`, 100%
speed limit, VSync **Immediate**, bilinear scaling, no AA or frame generation.
[Full system specifications, all 26 settings, raw samples and method](EDEN_BENCHMARK.md).

## Current release

Version **`0.1.1`** fixes the native Switch startup crash caused by missing inline
system-call register clobbers. The operator reports the corrected mod working on
Switch. Native Switch FPS has not been measured; online/desync compatibility has
not been established. The release remains experimental.

The October 9 refresh adds evidence and documentation; the binary and tag are
unchanged. [Startup diagnosis](SWITCH_BOOT_FIX.md).

| Artifact | Identity |
| --- | --- |
| Tag | `v0.1.1-experimental` |
| Binary source | `a534bb533bc0bd5b2fd0af0c518da55647337c44` |
| Installable file | `libgnw_render_probe.nro` — 528,384 bytes |
| SHA-256 | `bceef1f09e4339149cdab8e7826d17492680add02307002b0a5d62033d1090c8` |

## What it changes

G&W's stock vertex buffer includes many alternate poses and is 19,518,352 bytes.
The idle skinning shader writes only a small prefix, but Eden's accurate buffer
tracking downloads the entire declared writable range at a fence.

The NRO intercepts NVN compute recording. For one verified shader and stock
layout it temporarily narrows storage slot 2 to contain every written vertex,
dispatches the original command once, and immediately restores the original
address and size. It never changes the allocation, input data or vertex contents
itself. Unknown shader/layout/state/mapping combinations pass through unchanged.

It does not modify models, animations, skeletons, moves, timing, hitboxes,
hurtboxes, physics, RNG, game parameters, scripts or networking. That limited
scope is a design constraint, not proof of online safety.

Identity guards include the known control fingerprint and the full 4,600-byte
shader-code fingerprint, mapped memory validation, the 40 stock skinned objects,
buffer sizes, strides, constants and dispatch dimensions. Command-buffer state
is discarded at recording/lifetime boundaries and nested command streams.
Tracking and logging have fixed limits; exhausted tracking falls back unchanged.

## Install and roll back

Tested: Smash Ultimate **13.0.5**, Skyline with the existing ARCropolis loader,
Linux Eden, Vulkan, accurate GPU setting, stock assets. Exact loader and executable
hashes are in the [benchmark](EDEN_BENCHMARK.md). The release NRO has the experimental
range-probe feature enabled. Source builds are **log-only by default**.

1. Stop Eden completely. Identify the SDMC directory in the profile actually
   used by your Eden launcher. Do not assume a default path or overwrite saves.
2. Back up any existing file with the same name.
3. Copy `libgnw_render_probe.nro` to
   `SDMC/atmosphere/contents/01006A800016E000/romfs/skyline/plugins/`.
4. Retain the existing Skyline installation and ARCropolis. Disable other G&W
   render/skin/FPS experiments for the comparison.
5. Restart Eden. For diagnostic logs enable `Debug.Emulated:Info`. Look for
   `gnw-render-probe v4 RANGE_EXPERIMENT`, `shader code ... verified=true` and
   `RANGE ... restored=true`. Logs stop after 512 lines intentionally.

Rollback: stop Eden, move this NRO out of the plugin directory, restart.
No saves or settings need restoring for this plugin alone.

Do not enable the separate Eden `EDEN_GNW_DEMAND_READBACK` diagnostic while
testing this NRO. This project does not need a custom readback bypass.

## Build

Rust source, `skyline = 0.6.0`, and a committed `Cargo.lock`. Recorded tooling:
`cargo-skyline` commit `7d45fed93c8a4434bd3d03518fb9f4efcbf88e62`;
Skyline standard-library source `fabcf8e5bebc0d31e33717778a89df97d2b0e443`;
backing `nightly-2026-02-14`, custom toolchain `skyline-v3`.

Install cargo-skyline and its Skyline toolchain following
[the cargo-skyline project](https://github.com/ultimate-research/cargo-skyline).
Use isolated `CARGO_HOME` / `RUSTUP_HOME` to preserve an existing development setup.
Run each command separately from the configured project environment:

```powershell
/usr/bin/pwsh -NoLogo -NoProfile -NonInteractive -Command { $env:CARGO_BUILD_JOBS = '2'; cargo skyline build --release --no-default-features --features range-probe }
```

```powershell
/usr/bin/pwsh -NoLogo -NoProfile -NonInteractive -Command { $env:CARGO_BUILD_JOBS = '2'; cargo skyline clippy --no-default-features --features range-probe -- --all-targets -- -D warnings }
```

Output: `target/aarch64-skyline-switch/release/libgnw_render_probe.nro`.

Release builds retain optimized debug information in
`target/aarch64-skyline-switch/release/libgnw_render_probe.so`. Release assets
include that ELF separately, with source and SHA-256 hashes for crash analysis.
Install only `libgnw_render_probe.nro` in the Skyline plugins directory.

For an observation-only build omit `--features range-probe`. Host checks:

```powershell
/usr/bin/pwsh -NoLogo -NoProfile -NonInteractive -Command { cargo +stable test --no-default-features --features host-tests --jobs 2 }
```

```powershell
/usr/bin/pwsh -NoLogo -NoProfile -NonInteractive -Command { cargo +stable clippy --all-targets --no-default-features --features host-tests --jobs 2 -- -D warnings }
```

The corrected binary's build/test evidence is dated October 7. The October 9
refresh re-tested that same binary; it did not rebuild it. Historical builds used
4 GiB, no swap, two jobs and a 900-second watchdog. Current live runs used 8 GiB,
no swap, a 1,800-second limit and background Xwayland.

See [VALIDATION.md](VALIDATION.md) for evidence and limits, and
[YOGA_HANDOFF.md](YOGA_HANDOFF.md) for the requested desync test plan.

## Repository and evidence

[Forgejo](https://git.syntaxbyexample.com/intpa/gnw-render-fix) is authoritative;
[GitHub](https://github.com/crashbandicode/gnw-render-fix) is the private mirror.
Branches, tags and LFS objects synchronize from Forgejo to GitHub. Release objects
and assets are published separately on each service.

The release includes the install ZIP, exact NRO, matching debug ELF bundle,
source archive, dated benchmark bundle, screenshots, and `SHA256SUMS`.

No game assets, shader binaries, firmware, keys or saves are distributed.
