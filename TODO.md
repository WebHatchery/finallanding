# TODO — The Final Landing

## Standards migration

- [ ] Expose intentional game-logic APIs through `src/lib.rs`, make `main.rs` consume the library, and migrate all 52 `src/**/tests.rs` files plus test-only helpers into `tests/`. Remove test declarations from `src/`; preserve regression coverage without exposing private internals merely for tests (§11.4).
- [ ] Review migrated suites by feature, targeting at most five meaningful cases. Consolidate related inputs in Assign/Log models, resources, and summaries; document distinct coverage that warrants additional cases (§11.3).
- [ ] Move hardcoded balance and content from `data/`, `systems/`, and `game/colonist_spawner.rs` into typed JSON under `assets/`. Load through toolkit JSON APIs and validate IDs, references, costs, thresholds, and starting positions (§5.3).
- [ ] Externalize player-facing menu, toolbar, advisor, objective, and log text to JSON, preserving formatting parameters and loading it through the toolkit (§5.3).
- [ ] Move runtime `GameState` ownership into `state/` and remove `data/game_state.rs` dependencies on `BuildingSystem` and `TimeSystem`; keep reusable data types independent of orchestration (§2.1, §5.1).
- [ ] Split functions over 100 lines, including `draw_assign_context`, `draw_log_context`, and gameplay `draw`, into cohesive helpers; use context structs for long parameter lists and remove or narrowly justify the crate-wide Clippy allowances in `main.rs` (§4, §10.2).
- [ ] Add missing `//!` module-purpose comments and migrate the eight existing `mod.rs` roots to named module files when restructuring their domains (§2.3, §9.2).
- [ ] Correct README validation guidance to require parameterless `publish.ps1`, remove its nonexistent `game_state_helpers.rs` reference, and update `tests/code_standards.rs`'s obsolete “non-test lines” comment (§2.2, §8.3).
- [ ] Resolve capture executables from Cargo's target directory instead of the hardcoded `D:\WebHatchery\.cargo-target` layout in `scripts/capture_ui_smoke.ps1`; stop on a failed build instead of capturing an old executable.

## Browser controls and layout

- [ ] Add visible Undo, Cancel, and room-filter controls for the keyboard/right-click actions in `game_state_keyboard_input.rs` and `game_state_toolbar_input.rs`; make Log search usable with a touch keyboard (§7.5).
- [ ] Replace keyboard-only instructions in the menu, overlays, README, and `game_page.json` with exact visible control names. Add a tap-through introduction covering building, assignments, missions, and restart (§7.5).
- [ ] Make `ui/layout.rs`, toolbar panels, and `IsoView` responsive: fixed 300/302px rails and a minimum 520px toolbar currently crowd narrow viewports. Preserve reachable map cells and controls; extend smoke captures and the manual checklist to small desktop and touch layouts (§7.5, §12).
- [ ] Change menu and toolbar activation from `left_pressed` to toolkit release-based actions with cancellation; retain press handling only where immediate interaction is intentional (§7.4).
- [ ] Anchor build tooltips wholly above the build panel and render them after overlapping panels; `toolbar_tooltip_bounds` currently includes the panel itself. Provide tap-accessible details for hover-only information.
- [ ] Implement browser archive downloads for Log Export; `write_social_archive_markdown` currently always writes through `std::fs`. Reuse or extend toolkit export support and surface failures to the player.

## Remaining gameplay and presentation

- [ ] Enlarge the playable crash-site area and adjust camera framing and map picking together so expansion preserves readable terrain, buildings, and survivors.
- [ ] Add a new-run setup/arrival flow before `GameplayState::new` populates the six-survivor roster.
- [ ] Implement versioned save DTOs that preserve buildings, assignments, relationships, logs, and RNG state; current serialization skips buildings and RNG. Use this for a functional Continue menu action, then add Settings and platform-appropriate Exit controls.
- [ ] Replace procedural survivor portraits/poses and crash-site/building art with production assets; keep terrain, wreckage, buildings, and survivors visually distinct and include richer support/tension animation frames.
- [ ] Profile relationship summaries and Log filtering/rendering on a populated archive; cache demonstrated hotspots with invalidation on simulation, filter, and selection changes.
