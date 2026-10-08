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
- [ ] **Portrait variety.** Six painted portraits are reused with hue tints;
  larger crews repeat faces. Commission or generate more portrait art.
- [ ] **Pinch zoom.** Touch zoom currently uses the - / + buttons; add a
  two-finger pinch through the toolkit gesture helpers.
- [ ] **Audio.** No music or sound effects yet.
- [ ] **Painted species art.** Each of the 22 native species now has its own
  procedurally drawn node (`src/ui/world_view/species.rs`, reviewed in
  `docs/verification/ui_species.png`). Painted sprites in the building atlas
  style would sit better beside the painted buildings.
- [ ] **Ending balance.** Since survivors stopped sleeping beside predators and
  colonies prepare for winter, the scripted colony loses almost no one, and the
  Beacon future (which draws on hardship) no longer wins its matrix runs.
  Rootbound and Ascendant both do. Give Beacon another source of support.
- [ ] **Pacing.** Standard runs end at days 279–326 (≈24–28 hours before
  pauses). Pauses bring that to the 30–40 hour target only for players who
  stop often; consider a longer Act IV or V if playtests run short.
