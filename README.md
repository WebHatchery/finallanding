# The Final Landing

The Final Landing is a playable colony relationship manager about guiding a small crashed settlement by shaping where people live, work, recover, and connect.

You influence the colony through buildings, priorities, missions, role assignments, and space planning. Colonists follow daily routines, build relationships through proximity and shared work, and show pressure through visible tension/support signals.

## Current Status

Playable alpha loop:

- Isometric crash-site map with building placement and undo.
- Bottom toolbar modes for Build, Rooms, Objects, Colony, Research, Assign, and Log.
- Colony priorities for recovery, stockpiling, and surveying.
- Survivor routines for work, meals, sleep, movement, mission absence, and recovery.
- Relationship pressure driven by shared rooms, work locations, proximity, priorities, and directives.
- Assign mode for role cycling, room/work pins, pair/separate directives, roster filters, sorting, and batch copy controls.
- Research missions that return supplies, salvage, and technology progress.
- Daily social history, searchable Log mode, report drilldowns, and Markdown export.
- Day 7 victory/failure scenario checks backed by headless playthrough tests.
- Four-step arrival briefing, versioned autosave/Continue flow, and platform-aware exit/settings controls.
- Production crash-site, survivor, and building-atlas art layered with deterministic support/tension poses.

## Quick Start

Requirements:

- Rust stable toolchain.
- The sibling `macroquad-toolkit` crate at `..\macroquad-toolkit`, as referenced by `Cargo.toml`.

Run the native game:

```powershell
cargo run
```

Run tests:

```powershell
cargo test
```

Build only:

```powershell
cargo build
```

## How To Play

Goal: keep the settlement alive after the crash, stabilize food and shelter, recover field technology, and reach the Day 7 victory check as a functioning community.

Core loop:

1. Tap or click through the arrival briefing to learn the four decisions that shape the colony.
2. Place essential rooms and work objects.
3. Set the colony priority to match the current pressure.
4. Launch missions from Research once the Exploration Gate exists.
5. Use Assign mode to separate tense colonists or protect supportive pairs.
6. Watch the daily reports and adjust the plan before pressure becomes failure.

The settlement can limp or fail if it has no food plan, no habitats, or no mission/technology progress.

## Screen Brief and Viewport Contract

The normal 1280×720 canvas is the primary desktop composition. The current
minimum supported canvas is 720×480; smaller or portrait canvases are not a
support promise until they have a dedicated review. The game is touch-first at
both declared sizes, with keyboard shortcuts as supplements.

| Phase | Current decision | Dominant focus | Primary action | Supporting information | Deferred information |
| --- | --- | --- | --- | --- | --- |
| Arrival | Learn the next colony decision | Briefing copy and the visible Continue control | Continue / Enter Colony | Step count and the immediate mechanic | Full colony status |
| Observation | Decide which pressure to address next | The colony map and its visible people/buildings | Open one contextual mode or change priority | Current shortage, mood, and active warning | Full objectives, roster, archive, and research detail |
| Placement | Decide whether a plan fits the selected tile | Ghost footprint and affected map cells | Place, Undo, or Cancel | Cost, purpose, and blocking reason beside the plan | Secondary building catalog detail |
| Assignment | Decide who should work or live where | Selected survivor and compatible spaces | Role, Home, Work, Pair, or Apart | Mood, activity, injury, capacity, and tension | Batch tools and advanced filters |
| Research | Choose a mission tradeoff | Selected mission and its Launch action | Launch the selected mission | Crew, danger, duration, reward, and prerequisite | Technology effects until inspected |
| Reports | Understand what changed and what to do next | The selected daily story | Read, page, filter, or search | Recommendation beside the full report | Export and secondary metrics |
| Results | Review the outcome or begin another plan | Outcome summary and recovery choices | Review Log, Restart Run, or Return to Menu | The outcome condition and preserved colony context | Ongoing gameplay input |

The world area must remain the main space during observation. Context panels
replace that focus when opened; they must not stack with permanent objective,
resource, minimap, and roster dashboards. Menu, Settings, save recovery, and
exit controls remain separate from gameplay decisions.

## Controls

- Mouse or touch: release/click UI buttons, select colonists, place buildings, and pin compatible rooms/workspaces in Assign mode.
- Arrival briefing: use the visible Continue / Enter Colony button; the final step autosaves the new run.
- Bottom toolbar: switch between Build, Rooms, Objects, Colony, Research, Assign, and Log.
- `Q`, `W`, `E`, `R`, `T`: select Habitat, Mess Hall, Workshop, Storage, or Exploration Gate.
- `1`, `2`, `3`: set Recovery, Stockpile, or Survey priority.
- `Space`: pause or resume time.
- Top bar speed buttons: pause, normal, fast, or super fast.
- `Z`: undo last placement.
- `Esc`: cancel the current building/search interaction.
- `M`: launch the recommended mission when possible.
- `F3`: toggle debug overlay.
- Visible Undo and Cancel buttons mirror `Z` and `Esc`; the menu also exposes Continue, Settings, and Exit where supported.
- After victory/failure, `Enter`, `R`, or the visible Restart Run button starts another run.

Assign mode:

- Click a survivor card to cycle their work role.
- Click a compatible map building to pin or clear a selected survivor's recovery/work space.
- Right-click a room or work building, or arm the visible Filter Room control and tap a room, to filter the roster to survivors pinned there.
- Use relationship filters, role filters, sorting, and page controls to inspect pressure.
- Use `P-H`, `P-W`, `ALL-H`, and `ALL-W` to copy home/work pins across visible or full compatible rosters.
- Use pair/separate directives to encourage supportive pairs or keep tense colonists apart.

Log mode:

- Search, filter, and page daily social reports.
- Click a report row for a recommendation drilldown.
- On Windows, export the social archive to `docs\exports\social_archive.md`; in the browser, Export downloads `social_archive.md`.

## Major Systems

- **Buildings:** Habitat, Mess Hall, Workshop, Storage, and Exploration Gate define shelter, meals, salvage, storage, and mission access.
- **Resources:** food days, salvage, meals, survey, repair, hauling, and storage pressure are tracked in the right rail.
- **Relationships:** colonists gain support or tension from shared spaces, routines, directives, and colony priorities.
- **Missions:** field missions have duration, danger, cooldown, rewards, and technology chances.
- **Scenario:** the colony is evaluated around Day 7 for supplies, shelter, technology, and condition.
- **Advisor/objectives:** the left rail surfaces active risks and next priorities.
- **Arrival and persistence:** a guided first-run briefing precedes the six-survivor roster; Continue restores the latest versioned autosave.

## Implementation Notes

- Runtime and rendering use `macroquad`.
- `macroquad-toolkit` is used for pathfinding and toolkit colors.
- JSON balance, labels, and scenario content are loaded through toolkit configuration helpers and validated before play.
- Save slots use a versioned DTO that preserves runtime state, buildings, assignments, social history, and deterministic RNG state.
- Colonist movement uses `Grid::find_path`, not direct coordinate stepping.
- Simulation randomness is routed through deterministic `SimulationRng`, keeping domain behavior independent of `macroquad::rand`.
- UI rendering is split across focused modules:
  - `src\ui\top_bar.rs`
  - `src\ui\right_rail.rs`
  - `src\ui\toolbar_panel.rs`
  - `src\ui\toolbar_panel\assign.rs`
  - `src\ui\toolbar_panel\log.rs`
  - `src\ui\introduction.rs`
- Gameplay orchestration remains in `src\state\game_state.rs`, with pure helper logic kept in focused sibling modules.
- `Layout::responsive` and `IsoView` are refreshed from the live viewport so desktop resizing and touch-sized captures retain reachable controls and map cells.

## Verification

Primary checks:

```powershell
.\publish.ps1
```

Visual smoke captures:

```powershell
.\scripts\capture_ui_smoke.ps1
```

This captures gameplay screenshots at 1280x720, 1920x1080, and 720x480 touch-sized layouts plus key modes into `docs\verification\`.

Headless playthrough matrix:

```powershell
.\scripts\capture_playthrough_report.ps1
```

This writes `docs\verification\playthrough_report.md`. The current reference run reaches a Stable Victory at about 34.2 normal-speed minutes.

Focused manual pass:

- `docs\verification\manual_relationship_playtest.md`

Use it after changes to Assign mode, relationship logic, body-language rendering, Log mode, or restart flow.

## Capture Environment Variables

The native game supports deterministic capture helpers used by scripts:

- `TFL_START_GAMEPLAY=1`: skip the menu and start gameplay.
- `TFL_CAPTURE_PATH=path\to\image.png`: save a screenshot and exit.
- `TFL_CAPTURE_FRAMES=8`: choose how many frames render before capture.
- `TFL_WINDOW_WIDTH` / `TFL_WINDOW_HEIGHT`: set native capture size.
- `TFL_START_TOOLBAR_MODE=assign|log|research|...`: open a specific toolbar mode.
- `TFL_START_SELECTED_BUILDING=habitat|mess_hall|workshop|storage|exploration_gate`: seed a placement tool.
- `TFL_PREVIEW_GRID_X` / `TFL_PREVIEW_GRID_Y`: seed placement preview position.
- `TFL_START_SELECTED_COLONIST=id`: seed selected survivor.
- `TFL_SEED_ASSIGN_SPACES=1`, `TFL_SEED_SOCIAL_HISTORY=1`, `TFL_SEED_ACTIVITY_POSES=1`: seed verification fixtures.
- `TFL_PLAYTHROUGH_REPORT_PATH=path\to\report.md`: write the headless strategy matrix and exit.

## Playtest Checklist

- Start from the main menu without developer guidance.
- Read and complete the four-step arrival briefing; confirm Continue restores the active run from the menu afterward.
- Place at least one Habitat, Mess Hall, Workshop, Storage, and Exploration Gate.
- Change priorities after an incident and confirm the advisor/objective state responds.
- Launch at least two mission types from Research mode.
- Use Assign mode to retask one survivor, pin a Habitat and work space, filter by a right-clicked room, and copy pins across the visible roster.
- Confirm tense/supportive survivors show visible map markers and useful inspector details.
- Open Log mode after several day summaries; search, filter, page, select a report, and export the archive.
- Reach victory or failure, then restart and confirm the new run does not leak old selection/log state.
- Repeat the first-frame, Assign, and Log checks at 720x480 using taps and the visible controls.

## Documentation

- `tfl_mvp.md` — the design document: the single pillar, MVP scope, and the loop the game is built around.
- `docs/verification/manual_relationship_playtest.md` — focused manual QA pass.
- `docs/verification/playthrough_report.md` — generated headless strategy matrix.
- `TODO.md` — only remaining AI-agent tasks, if any.
