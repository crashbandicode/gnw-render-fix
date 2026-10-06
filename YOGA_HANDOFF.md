# Yoga handoff — G&W Rust NRO and desync validation

The operator requested that this NRO be staged with the Eden CLI handoff and
that the receiving agent **test for desync**. It has not been installed in the
active Yoga profile. Existing Eden settings, ROMs, firmware, keys and saves must
remain in use; do not replace them with the investigator's isolated profile.

This is a bounded validation task. The original investigator retains the wider
G&W investigation. The separate TAS DSL tangent is unrelated and awaits its
operator's go.

## Contents and known result

libgnw_render_probe.nro is the range-enabled Rust Skyline build.
source/ contains the independent project; SHA256SUMS identifies staged files.
The private GitHub repository/release URLs are recorded in DELIVERY.md.

Four idle stock G&Ws improved from 28.00 to 60.00 FPS locally without an FPS mod.
Read README.md and VALIDATION.md before testing. **Online compatibility is
unverified** despite the plugin's rendering-only scope.

## Profile and installation

Use the existing Eden launcher/profile on Yoga and locate its configured SDMC.
Record the actual executable, game/update version, renderer, accuracy, mods and
profile paths. Prefer a copy of that profile for the first offline A/B so saves
and the user's current work are unaffected. Keep the same ROM and update paths.

Stop the target Eden instance before touching plugins. Back up any existing
libgnw_render_probe.nro. The Skyline plugin path relative to SDMC is:
atmosphere/contents/01006A800016E000/romfs/skyline/plugins/

Place only this NRO there; retain existing Skyline/ARCropolis. Disable competing
G&W/FPS/render experiments for the baseline. Do not turn on
EDEN_GNW_DEMAND_READBACK. Enable Debug.Emulated:Info to confirm v4, shader
verified=true and RANGE/restored=true. If those are absent, do not attribute
a speedup to the plugin. Log cap 512 is intentional.

Use the delivered Eden socket CLI for menus/input/screenshots. Use a private
display or existing background controls; never take over the physical desktop.
Use a finite watchdog, no swap, and an 8 GiB memory limit if current available RAM
allows. Do not run build or GPU replay tasks alongside the game.

## Required tests before any online-safe claim

1. Offline A/B: same settings, stock G&W, one/two/four fighters in Training.
   Warm each scene; record FPS/frame time and screenshots for idle, Chef, jab,
   taunts, grabs/throws, air actions and result/victory poses. Include all 8
   costumes when practical, a second stage, another character as an opponent,
   and 8 G&W as a separately bounded stress test only if memory allows.
2. Establish a control match with this NRO absent on both peers. Use a willing
   controlled test peer/private lobby, not public matchmaking. Record identical
   game update/DLC, other mods, speed settings and network mode.
3. Repeat with the NRO on **one client only**, keeping the other unmodified.
   This asymmetric case is essential for a client-side mod. Exercise idle and
   the above moves, damage/KO/respawn, grabs, shields, items if enabled, and
   match completion/rematch. Observe both peers for divergent positions, damage,
   stocks, timers, disconnects or explicit desync messages.
4. If supported, compare simulation-state/checksum traces or deterministic input
   replay against the control. Matching screenshots or one successful match
   alone do not prove absence of desync.
5. Repeat several matches with roles reversed; then both clients with the NRO.
   Log game/build/mod versions, durations, outcomes and any mismatches.
6. On any mismatch/crash/artifact: stop, preserve logs and exact inputs, disable
   the NRO, and reproduce against control. Do not change gameplay/network logic
   to make a test pass.

Report what was actually tested and retain “not yet verified” for everything
else. If no suitable peer is available, stop at offline results and explicitly
leave desync validation pending. Do not claim certification.

## Rollback

Stop Eden, remove/move only this NRO from Skyline's plugins directory and
restart. Restore any same-name plugin backup. Do not delete game data or rewrite
the profile. Keep the staged handoff and measured evidence.
