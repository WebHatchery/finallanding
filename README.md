# The Final Landing

A colony simulation of autonomous minds. The colony ship *Meridian* has broken
apart over an unknown world; eight survivors crawl from the wreck. You are
MERIDIAN, the ship's intelligence. You cannot command anyone. You shape the
world they live in — what gets built, what the colony asks for, what it
researches, where expeditions go and, eventually, which future the colony
votes on — and you watch them think, love, quarrel and choose.

`GAME_DESIGN.md` holds the full design: the review of the original MVP, the
five-act campaign, the dynamic technology tree, and the Prometheus agent
design behind the survivors.

## What is in the game

- **Survivors as BDI agents (Prometheus methodology).** Each survivor perceives
  only what is near them, keeps beliefs that can go stale, generates goals from
  needs, personality, skills and ambitions, and chooses among several plans per
  goal. Failed plans correct beliefs and trigger another plan; exhausted goals
  back off and can become a help request. Survivors exchange messages: warnings,
  knowledge, gossip, help requests, courtship.
- **Relationships with consequences.** Opinions grow from conversation, shared
  work and help; trait clashes and tempers produce arguments, fights, rivalries
  and reconciliations. Partners move into family quarters and raise children.
  Grief, breakdowns and departures follow neglect.
- **A technology tree grown from what each colony finds.** Every landing draws
  its own native foods, fibres, stones, ores and ruins (22 species in all) and
  scatters them in territories, so each crew gathers different things in a
  different order. Gathering a species inspires the technologies it suggests;
  technologies tied to species this world lacks never appear. The crew's own
  knowledge is re-wired from the seed as well: prerequisites change, some starts
  hidden until a eureka, forks lock alternatives, and practice makes a branch
  cheaper. Orchards grow whichever native food the colony gathers most.
- **A five-act campaign** (Landfall, Roots, Echoes, Divergence, The Final
  Landing) over five in-game years with three endings — Beacon, Rootbound and
  Ascendant — chosen by a colony vote that can divide it.
- **Replayability:** seeded maps and native species, three landing sites, three
  difficulties, generated crews, a tree that grows from each run's finds, a
  36-card event deck and three endings.
- **Persistence:** daily autosave through the toolkit (native files or browser
  storage); Continue resumes the run.

## Quick start

Requirements: Rust stable and the sibling `macroquad-toolkit` crate at
`..\macroquad-toolkit`.

```powershell
..\rust_management\cargo.ps1 run --release
..\rust_management\cargo.ps1 test
..\rust_management\cargo.ps1 clippy '--' -D warnings
.\publish.ps1
```

## Viewport contract

The game is composed for a fixed **1920×1080** virtual canvas and letterboxed
to the window. 1920×1080 is the normal size; **1280×720** is the minimum
supported window (the canvas renders at 0.67 scale). Touch controls are at
least 44 virtual pixels tall; buttons fire on release; every action has a
visible control and keyboard shortcuts are only supplements.

## Screen brief

| Phase | Current decision | Dominant focus | Primary action | Supporting information | Deferred information |
| --- | --- | --- | --- | --- | --- |
| Title | Start, continue or quit | Crash-site art and the menu | New Colony / Continue | Save presence | Settings |
| New colony | Choose site, difficulty and seed | Site cards | Land | Climate, plentiful and scarce resources, storms, ruins | Crew until landing |
| Observation | Which pressure to address next | The colony map | Open a tool or tap something | Resources, date, weather, act objectives, alerts | Roster, tech tree, relations, chronicle |
| Placement | Does this building fit here? | Ghost footprint | Tap to place / Close | Cost and blocking reason at the pointer | Other categories |
| Inspection | Why is this survivor doing that? | Inspector beside the map | Close or Follow | Goal, reason, plan steps, reasoning log, beliefs vs. reality | Needs, bonds, life story in tabs |
| Research | Which technology next | Known technologies, laid out from what was found | Set as research focus | Cost, progress, inspiring species, prerequisites, unlocks, forks, native finds | Hidden and absent technologies |
| Expeditions | Where to send people | Site list | Call for volunteers | Danger, duration, yield, rewards | Results until return |
| Relations | Who is close or in conflict | Relationship web | Tap a survivor | Two-sided opinions | Thought history |
| Chronicle | What happened and why | Daily reports and entries | Filter / page | Category, day | Older entries via paging |
| Council (Act IV) | Which future to propose | Three ending cards | Propose this future | Projected support, champions, opponents | Capstone details |
| Results | Review the ending | Epilogue and fates | New Colony / Title | Colony record and highlights | — |

Layout at 1920×1080: a 56 px top bar (date, weather, resources, power,
population, mood, speed, help, menu) and a 76 px toolbar (Build, Colony,
Research, People, Relations, Expeditions, Chronicle, Council; zoom and
recenter) frame the map. The act tracker and alerts sit at the upper left;
the inspector opens at the right only while something is selected; tools
open as overlays that replace the map focus.

## Controls

- **Tap / click** a survivor, building, resource site or creature to inspect it;
  tap empty ground to dismiss.
- **Drag** the map to pan; **scroll** or the toolbar **- / +** to zoom;
  **Recenter** returns to the crash site.
- **Build:** tap Build, pick a category and a building, then tap the map.
  Close (or Esc / right-click) disarms the tool.
- Speed buttons **II > >> >>>** (Space, 1, 2, 3). Help (F1) and Menu (saves
  and returns to the title) sit at the top right.
- Keyboard supplements: WASD / arrows pan, Q / E zoom, B build, C colony,
  R research, P people, L relations, X expeditions, J chronicle, Esc cancel.

## Architecture

- `src/data/` — typed schemas, the embedded JSON catalog (`assets/data/*.json`
  loaded with `macroquad_toolkit::include_json!`, including the native species
  in `finds.json`) and semantic validation.
- `src/world/` — tile map, structures, landscape features, fauna, weather,
  calendar and seeded map generation.
- `src/agents/` — the BDI survivor: personality, needs, beliefs, goals,
  plans, intentions, messages, deliberation and the plan library.
- `src/colony/` — stockpile, priorities, policies, the species the colony has
  found, the per-run tech tree, research, expeditions, campaign state,
  chronicle and the job board.
- `src/sim/` — the tick: perception, inbox, cognition, step execution,
  movement, social outcomes, environment, fauna, director, expeditions,
  campaign and the Divergence vote. `sim::commands` is the player's only lever.
- `src/state/`, `src/game.rs` — screens, play state, action dispatch, autosave
  and capture scenes.
- `src/ui/` — immediate-mode UI on the toolkit's `VirtualUi` and `Pointer`;
  UI code returns `UiAction`s and never mutates the simulation.
- `src/autoplay.rs` — a scripted colony AI that plays through the same
  commands, used for the pacing report and tests.

Simulation randomness uses the run's state-owned `SeededRng`; the same seed
replays identically (covered by tests).

## Verification

- `cargo test` — data coverage, BDI agent properties, tech tree, native
  species and inspiration, campaign, simulation (determinism, save round-trip,
  expeditions, a scripted colony's first weeks), painted assets standing on
  their footprints, interface geometry and the 800-line source gate.
- `.\scripts\capture_ui_smoke.ps1` — every scene at 1920×1080 (including an
  `assets` gallery of every building, landscape feature and creature) plus the
  busiest scenes at 1280×720, written to `docs\verification\ui_<scene>.png`.
- `.\scripts\capture_playthrough_report.ps1` — headless campaign matrix at
  `docs\verification\campaign_report.md`.
- `docs\verification\manual_playtest.md` — the hands-on checklist.

Capture environment variables: `TFL_CAPTURE_MANIFEST` (scene⇥path rows),
`TFL_CAPTURE_FRAMES`, `TFL_WINDOW_WIDTH` / `TFL_WINDOW_HEIGHT`,
`TFL_CAMPAIGN_REPORT_PATH`, `TFL_CAMPAIGN_MAX_DAYS`.
