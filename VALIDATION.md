# Validation

## October 9, 2026: corrected binary performance re-verification

The existing `v0.1.1-experimental` NRO was re-tested without rebuilding it.
Four stock G&Ws idle on the Training stage measured **28.00 → 59.99 → 29.00
median FPS** in disabled / enabled / disabled order. Each warmed window contains
60 one-second observations; all reported zero shaders compiling. Effective
settings and executable hashes matched; the exact plugin inventories were checked.

The enabled minimum observation was 41.99 FPS, so the result is not a locked-60
claim. Stock extra-CPU palette assignments varied between launches. All measured
frontends exited with code `0`; the original plugin set was restored.

See [the full benchmark](EDEN_BENCHMARK.md) for unretouched before/after/control
screenshots, system specifications, effective per-game settings, raw measurements,
exact hashes, resource counters, and limitations.

The operator now reports the corrected mod working on physical Switch. This is
operator-reported functionality, not an instrumented hardware FPS result or
multiplayer compatibility finding.

## Version 0.1.1: Switch startup correction

The corrected range-enabled NRO has SHA-256:

```text
bceef1f09e4339149cdab8e7826d17492680add02307002b0a5d62033d1090c8
```

Its matching optimized debug ELF has SHA-256:

```text
e4448f2fdc98ad583b7bb355a09f1f802fe901c5a76697823b4af605fe572232
```

Validation on 2026-10-07:

- Four host bounds tests passed. Host Clippy for all targets, host build without
  default features, Switch-target Clippy with warnings denied, and the optimized
  Skyline build with `range-probe` passed.
- Disassembly of the exact NRO confirms the shader hook preserves its live
  pointer and range boundary across QueryMemory. Both source SVC sites declare
  the complete caller-saved clobber set. Host tests alone do not cover this code.
- Eden CLI build `10bcd2d849843b146a79c61a21df615e1bdcfcde`, Smash 13.0.5,
  Vulkan/accurate, docked, 100% speed: startup, character select, and Training as
  stock Mr. Game & Watch versus Zero Suit Samus passed. Idle, Chef, and jab were
  visually inspected, then Training returned to character select without a crash.
- The same run loaded Online Deluxe rewrite `4a927dc20904722d`, Training Modpack,
  the corrected gamepad overlay, Elite Legal Stages, and standalone ImGui. The
  overlay was visible. GNW logs showed the exact shader fingerprint verified and
  `RANGE ... restored=true`. No separate Eden demand-readback bypass was enabled.
- This is a targeted boot/render regression check, not a new speedup measurement,
  exhaustive pose test, or multiplayer compatibility certificate.
- Native Switch retry was pending at the October 7 checkpoint; the October 9
  operator report above supersedes that status. The old binary's three matching
  hardware reports and root cause are in [SWITCH_BOOT_FIX.md](SWITCH_BOOT_FIX.md).

## Original v4 / 0.1.0 measurements (historical)

This is offline validation, not an online compatibility certificate.

## Build and measurements

- Rust/Skyline cross build: release, no default features, range-probe enabled.
- Cross Clippy: all targets, warnings denied, passed.
- Host bounds tests: four passed, covering all 40 stock objects, mismatches and
  overflow. Host Clippy all targets with warnings denied passed.
- Exact NRO SHA-256:
  0883e69785e90b0e6324a803821e82a8cf4df529e6200c25c1e9658069620ade.
- Linux RTX 3050 Ti 4 GiB; system performance power profile. Same private display,
  Eden build/configuration and isolated Smash 13.0.5 profile.
- Four stock G&Ws idle: log-only baseline median 27.99788 FPS over 26.36s;
  range NRO median 59.99556 over 25.81s (min 59.00617, max 60.05319).
- No FPS mod, replacement skin or Eden diagnostic readback flag.
- Stock silhouettes/heads/hands visible in two- and four-fighter scenes.
  Chef and return-to-idle smoke checks passed. A suspected missing hand in one
  screenshot was rejected by independent image review; both hands are present.
  This does not exhaustively test every pose, costume, stage or mod interaction.
- Bounded game runs closed via Eden's CLI, without OOM. The measured run peaked
  at 6.25 GiB inside its 8 GiB/noSwap cgroup. No system keyboard/mouse input.
- Fable 5.1/xhigh independently reviewed the hook and screenshots.

## Why the shader write bound is 28 bytes per vertex

The verified shader has code fingerprint FNV64 3869d88ad69eee6a over 4600 B.
It matches the serialized Eden compute pipeline c478aab12efe386f, local size
(256,1,1). Only fingerprints are included, not game code.

Let t=base+28*vertex, with vertex<count enforced by the shader.
The position stores are t+0,t+4,t+8 (32-bit words). Their Phi operands preserve
these offsets; they do not accumulate in the bone loop.

The NRO requires c3[5]=2 and c3[8]=c3[9]=1. Thus the second loop runs twice and
takes the packed-attribute path both times. It starts at t+12, writes two words,
advances 8, writes two more at t+20 and t+24, then exits. The unpacked paths are
not taken for the guarded constants. The last word ends at t+28.
Stock buffer addresses are aligned; the NVN address adjustment contributes zero
low bits in the validated layout.

The exclusive end is therefore base+28*count, rounded up to 256 bytes.
The largest allowed stock object ends at 739956 B, rounded 740096 B.
The original address, data and final full-size binding remain unchanged.

## Remaining limitations

- Online/desync testing is explicitly pending; use a controlled peer session.
- No native Switch speedup was measured. The geometry itself is unchanged.
- Only the observed stock layout/shader is supported; other layouts fall back.
- The 64-command-buffer limit was reached once during loading. Overflow leaves
  original commands untouched and can reduce optimization coverage.
- CPU-mapped constants are inspected at recording time. The GPU capture audit
  checks them against execution-time constants for the captured scene; it is
  not a proof for every future workload.
- This remains an opt-in prototype. A general Eden readback/coherence correction
  could address this cost without a game-specific plugin.

## GPU capture audit

All 48 dispatches in the captured four-fighter idle sequence had the verified
shader and matching execution-time c3 constants. Every narrowed output descriptor
contained the calculated writes and equalled the 256-byte-rounded end. All output
address low four bits were zero. The twelve per-object bounds ranged from 312832
to 630272 bytes, repeated for four fighters.

The four output-to-staging copies decreased from 4 x 19518352 = 78073408 bytes
to 4 x 630272 = 2521088 bytes: 96.77% fewer copied bytes for this workload.
This confirms the intended mechanism independently of the FPS sample.

The range capture contains all 48 compute dispatches but only three graphics
draw actions, versus 349 in the older stock capture. Therefore those captures
are used for compute/range/readback validation, NOT total-frame GPU timing or
draw-count equivalence. Live screenshots and the separate FPS run are the visual
and performance evidence. A broader visual/state regression matrix remains open.

Capture replay completed in 13.37s at 492 MiB, inside 6 GiB/noSwap/180s.
The capture game run closed cleanly at a peak of 6.51 GiB, without OOM.
