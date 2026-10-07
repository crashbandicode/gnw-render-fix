# Switch startup crash fixed in 0.1.1

The 0.1.0 experimental NRO could close Smash before the title screen on a real
Switch. Three hardware reports recorded the same MainThread Data Abort:

```text
Result: 2168-0002
PC: gnw_render_probe + 0x4c1c
Address: 0x128
X8: 0
```

The affected binary has SHA-256:

```text
0883e69785e90b0e6324a803821e82a8cf4df529e6200c25c1e9658069620ade
```

## Cause and correction

The shader inspection hook calls Horizon QueryMemory through inline assembly.
Its old assembly declaration omitted the caller-saved register clobbers. The
optimizer therefore kept a vector pointer in X8 and a range boundary in X9 across
the system call. On the reported Switch runs those registers were cleared. The
next load dereferenced X8 plus 0x128; the preceding range check also used stale X9.

The fix declares `clobber_abi("C")` at both QueryMemory and OutputDebugString and
makes consumed argument registers explicit input/output operands. This fixes the
compiler contract instead of adding a null-pointer exception to the shader path.
See the [Rust inline assembly contract](https://doc.rust-lang.org/reference/inline-assembly.html#abi-clobbers)
and the [libnx system-call wrappers](https://github.com/switchbrew/libnx/blob/master/nx/source/kernel/svc.s).

The optimized 0.1.1 NRO keeps the shader pointer and range boundary in callee-saved
registers X22 and X21 across QueryMemory, and reloads mapping data afterwards.
OutputDebugString also has the complete clobber declaration. The rendering
algorithm, shader fingerprints, bounds, dispatch, restoration, and default
log-only feature selection are unchanged. Release NROs still enable `range-probe`.

The old binary passed Eden tests. Those tests did not expose this hardware
register-clobber failure; successful emulator execution alone cannot validate a
Switch system-call contract. A hardware retry remains the final boot check.

## Installation and rollback

Close Smash before replacing the NRO. Keep the previous file outside Skyline's
active plugin directory, then install the new file at:

```text
atmosphere/contents/01006A800016E000/romfs/skyline/plugins/libgnw_render_probe.nro
```

No INI, save, effect mod, Skyline, or ARCropolis change is required by this fix.
To isolate a later problem, close Smash and move this NRO outside the plugins
directory. Do not reinstall the known crashing 0.1.0 binary on this Switch.

Test startup, a match with Mr. Game & Watch, and return through results to
character select. Preserve any new Atmosphere crash report and the exact NRO hash.
This correction does not establish a native Switch performance benefit or certify
all multiplayer combinations.
