# G&W render fix — Rust Skyline NRO

Version **0.1.1** fixes a Switch startup crash in the plugin's memory inspection.
Both inline system calls now declare the registers they can overwrite. The
render-range optimization and its guards are unchanged. See
[the crash diagnosis](SWITCH_BOOT_FIX.md) and [validation](VALIDATION.md).

An experimental client-side rendering fix for Mr. Game & Watch in Super Smash
Bros. Ultimate under Eden. The private prerelease is for controlled testing.
**Online/desync compatibility has not been established.**

Four stock G&Ws idling in Training improved from **28.00 to 60.00 FPS** on the
test machine, without an FPS mod or replacement skin. This targets an Eden
readback cost; no native Switch performance benefit is claimed.

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

## Install for an isolated test

Tested: Smash Ultimate **13.0.5**, Skyline with ARCropolis **4.1.1**, Linux Eden,
Vulkan, accurate GPU setting, stock assets. The release NRO has the experimental
range-probe feature enabled. Source builds are **log-only by default**.

1. Stop Eden completely. Identify the SDMC directory in the profile actually
   used by your Eden launcher. Do not assume a default path or overwrite saves.
2. Back up any existing file with the same name.
3. Copy libgnw_render_probe.nro to:
   SDMC/atmosphere/contents/01006A800016E000/romfs/skyline/plugins/
4. Retain the existing Skyline installation and ARCropolis. Disable other G&W
   render/skin/FPS experiments for the comparison.
5. Restart Eden. For diagnostic logs enable Debug.Emulated:Info. Look for
   “gnw-render-probe v4 RANGE_EXPERIMENT”, “shader code ... verified=true” and
   “RANGE ... restored=true”. Logs stop after 512 lines intentionally.

Rollback: stop Eden, move this NRO out of the plugin directory, restart.
No saves or settings need restoring for this plugin alone.

Do not enable the separate Eden EDEN_GNW_DEMAND_READBACK diagnostic while
testing this NRO. This project does not need a custom readback bypass.

## Build

Rust source, skyline = 0.6.0, and a committed Cargo.lock. Tested tooling:
cargo-skyline commit 7d45fed93c8a4434bd3d03518fb9f4efcbf88e62;
Skyline std source fabcf8e5bebc0d31e33717778a89df97d2b0e443;
backing nightly-2026-02-14, custom toolchain skyline-v3.

Install cargo-skyline and its Skyline toolchain following
[the cargo-skyline project](https://github.com/ultimate-research/cargo-skyline).
Use isolated CARGO_HOME/RUSTUP_HOME to preserve an existing development setup.
Set CARGO_BUILD_JOBS=2; this wrapper does not accept a top-level --jobs argument.

    cargo skyline build --release --no-default-features --features range-probe
    cargo skyline clippy --no-default-features --features range-probe -- --all-targets -- -D warnings

Output: target/aarch64-skyline-switch/release/libgnw_render_probe.nro.

Release builds retain optimized debug information in
`target/aarch64-skyline-switch/release/libgnw_render_probe.so`. Release assets
include that ELF separately, with source and SHA-256 hashes for crash analysis.
Install only `libgnw_render_probe.nro` in the Skyline plugins directory.

For an observation-only build omit --features range-probe. Host bounds tests:

    cargo +stable test --no-default-features --features host-tests --jobs 2
    cargo +stable clippy --all-targets --no-default-features --features host-tests --jobs 2 -- -D warnings

On native Ubuntu in this workspace, invoke commands through
/usr/bin/pwsh -NoLogo -NoProfile -NonInteractive -Command ...
Builds were limited to 4 GiB, no swap, two jobs and a 900-second watchdog.
Live tests used 8 GiB, no swap, a 900-second watchdog and a private X11 display.
Do not compile or replay a GPU capture alongside the live emulator.

See [VALIDATION.md](VALIDATION.md) for evidence and limits, and
[YOGA_HANDOFF.md](YOGA_HANDOFF.md) for the requested desync test plan.

No game assets, shader binaries, firmware, keys or saves are distributed.
