# TODO — The Final Landing

Remaining agent tasks after the 2026-10 rebuild (colony of BDI survivors,
dynamic tech tree, five-act campaign, 1920×1080 UI).

- [ ] **Run `.\publish.ps1` on the Windows workspace.** The rebuild was made in a
  Linux cloud container without `rust_management` or PowerShell; native and
  `wasm32-unknown-unknown` release builds, fmt, Clippy (`-D warnings`) and tests
  passed there, but the shared publisher has not run.
- [ ] **Browser touch pass.** Verify tap, drag-to-pan, the build tool and every
  overlay in the embedded WebGL page at 1920×1080 and 1280×720, including the
  first-run terrain bake time on slower machines.
- [ ] **Delete `legacy/`.** The previous implementation was moved there instead
  of deleted; it is not compiled. Remove it once the rebuild is accepted.
- [ ] **Portrait variety.** Six painted portraits are reused with hue tints;
  larger crews repeat faces. Commission or generate more portrait art.
- [ ] **Pinch zoom.** Touch zoom currently uses the - / + buttons; add a
  two-finger pinch through the toolkit gesture helpers.
- [ ] **Audio.** No music or sound effects yet.
