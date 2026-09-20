# TODO — The Final Landing

## Previous status

Before the 2026-09-20 audit this file recorded “No remaining AI-agent tasks.”
There were no existing task checkboxes or completion entries to merge or remove.

## UI_STYLE review — 2026-09-20

The audit below is the implementation backlog. Completed, verified items are
removed as each cohesive change lands. Paths below are relative to the project
root. Keep this existing tracked `TODO.md` as the single backlog (the requested
`todo.md` resolves to it on Windows).

### Evidence and scope

- Read `AGENTS.md`, `UI_STYLE.md`, `CODE_STANDARDS.md`,
  `GAME_DEVELOPMENT_GUIDE.md`, `README.md`, and `tfl_mvp.md`. No project-local
  `PROJECT_AGENTS.md` was found. Reviewed rendering, hit zones, state transitions,
  pointer routing, camera math, capture code, configuration text, and the existing
  manual relationship checklist.
  - **Visual evidence:** the current checkout has fresh native captures in
    `docs/verification/` for ordinary, closed-tray, Colony objectives, Assign,
    Research, placement, activity poses, Log timeline, event history, selected Log
    report, menu, results, help, and the 720x480 Assign/Research states. The normal 1280x720 and large
  1920x1080 captures show the colony as the dominant field without permanent
  rails; the Colony, Assign, Research, and Log captures show secondary information
  replacing the world area instead of stacking over it. These captures verify
  composition and readability, not live touch behavior. The toolkit-tooltip image
  has older art/layout and artificial text; do not treat its test copy as shipped
  demo prose. `tfl_guide_mvp.png` and `catalog_thumbnail.png` are not gameplay QA
  evidence for this review.
- **Code evidence** below means a source-confirmed layout or behavior, not a claim
  that a live mouse/touch path was exercised. The original audit did not launch
  the game, overwrite screenshots, perform a browser touch test, or run
  publishing. The implementation slice now has fresh native captures and focused
  hit-zone regression coverage; live browser/touch checks stay attached to UI-04
  and UI-10.
- Use 1280x720 as the normal baseline, 1920x1080 as the large check, and 720x480 as
  the current documented touch-sized baseline. README lists these capture sizes
  and README now explicitly declares 720x480 as the current minimum touch-sized
  baseline; narrower and portrait canvases remain unverified.
- Preserve the game's spatial relationship loop, building costs, urgent survival
  warnings, mission risks, autosave recovery, and visible Undo/Cancel/Restart.
  There is no evidence that the whole UI was copied from the template; the fixed
  inspector meters are specific demonstration-style remnants confirmed below.

### Verified findings — implement in this dependency order

- [ ] **UI-03 / P1 — Fit the camera to unobstructed space at a selectable scale.**
  - **Screen/files:** observation, placement and room selection;
    `src/ui/isometric.rs::IsoView::for_area`, `src/ui/layout.rs::game_area`,
    `src/ui/gameplay/{terrain,picking,colonists,placement_preview}.rs`,
    `src/state/game_state_pointer_bounds.rs`, `src/state/game_state_queries.rs`.
  - **Observed (screenshots + code):** fitting the whole diamond into the space
    between permanent rails compresses survivors and overlapping HOME/WORK labels
    at 720x480. `IsoView` fits against `game_area`, which excludes the toolbar but
    not the 126px context tray; rendering and blocked map space differ. There is
    no player camera zoom/pan path in the reviewed code. At 1920x1080 the scene is
    larger, so do not describe every viewport as equally cramped.
  - **Change:** after UI-01, compute the actual unobstructed world rectangle and
    frame useful colony activity within it. Provide bounded pan/zoom with visible
    zoom and recenter controls if fitting the whole map makes targets too small;
    reuse toolkit camera/viewport helpers. Keep touch placement distinguishable
    from dragging. Show selected survivor/room labels contextually and resolve
    their overlaps; keep selection and warning outlines visible over wreckage.
  - **Acceptance:** survivors, room footprints and available cells are easy to
    distinguish and select; every valid map area remains reachable; opening a
    tray never strands targets under it. Rendering, preview and picking use the
    same camera transform and active UI bounds.
  - **Verify:** all three sizes, open/closed context, occupied and edge cells,
    clustered survivors, resize, zoom and display scaling. Use taps and drags to
    place, cancel, inspect, pin and recenter; confirm dragging never places a room.
  - **Current state:** rendering, placement preview, picking, and blocked-world
  input now share the unobstructed world rectangle, with bounded visible zoom,
  recenter, and drag-to-pan controls. Remaining work is live resize/touch
  verification.

- [ ] **UI-04 / P1 — Reflow controls and consume input in the visible topmost layer.**
  - **Screen/files:** top bar, Assign filters, Log search/keyboard, modal states;
    `src/ui/hit_zones/{top_bar,assign,log}.rs`, `src/ui/top_bar.rs`,
    `src/ui/toolbar_panel/{assign,log}.rs`,
    `src/state/game_state_{lifecycle,toolbar_input,pointer_bounds}.rs`,
    `src/state/game_state_{building_commands,map_selection,assign_filter_commands}.rs`.
  - **Observed (code, with screenshot overlap evidence):** phone speed controls
    are 36x20, Assign controls commonly 40x17, and Log rows only 13px high. At
    1280px Undo starts at x=1113 while Survey ends at x=1129, so they overlap and
    Undo wins dispatch. Armed Filter Room returns before top-bar handling, even
    for a Cancel tap. Pointer blocking omits the Log keyboard above its tray,
    and selection/placement run after UI handling using the same event. These
    paths require runtime checks; do not claim every gesture currently fails.
  - **Change:** use the UI-01 space budget to reflow into fewer readable rows or
    deliberate scrolling, not smaller text. Target at least 44x44 logical tap
    areas, then verify their actual browser size. Place Undo/Cancel beside the
    relevant interaction with clear enabled states. Route a normalized pointer
    through active overlay/UI/world once, consume handled releases, let Cancel
    disarm room filtering everywhere, and block the world beneath keyboard and
    report overlays. Use the same geometry for drawing and hit testing.
  - **Acceptance:** labels and controls do not overlap, each tap causes one
    intended action, and dismissal always works without a keyboard. Touch targets
    remain usable after embedding/scaling and no overlay tap changes the colony.
  - **Verify:** 720x480, 1280x720, and immediately around the 760/1100 breakpoints.
    Tap Survey versus Undo; arm Filter Room then Cancel; type/erase/finish a Log
    search above survivors; change modes and resize. Preserve useful hit-zone
    regressions and add focused routing cases in `tests/`. Depends on UI-01/03.
  - **Current state:** top-bar collisions are removed, Assign inspection/actions
    use separate hit zones, report dismissal is topmost, and handled releases no
    longer fall through to world selection. Remaining work is 44px logical target
    sizing at compact breakpoints plus live browser/touch verification.

- [ ] **UI-05 / P1 — Make survivor inspection truthful and separate selection from commands.**
  - **Screen/files:** selected survivor, right roster and Assign;
    `src/ui/colonist_inspector.rs::draw_colonist_inspector`,
    `src/ui/right_rail.rs`, `src/ui/toolbar_panel/assign.rs`,
    `src/state/game_state_assign_roster_commands.rs::update_assign_click`,
    `src/ui/gameplay/picking.rs::inspected_colonist`, `src/data/colonist.rs`.
  - **Observed (code + saved selected-state screenshots):** Energy=0.46 and
    Hunger=0.58 are constants, Health is an arbitrary 0.45/0.82 injury proxy.
    These look like simulated needs. Inspector height shrinks but its portrait,
    fixed-width bars and text offsets do not. Assign card taps mutate a role or
    relationship directive depending on prior selection; hovering can replace
    the inspected survivor. The visible right roster has no selection handler.
  - **Change:** remove invented Energy/Hunger/Health meters; show actual mood,
    activity, role, room pins and injury recovery. Make tapping a survivor select
    and open a dismissible responsive inspector without issuing a command; use
    explicit labeled Role, Pair/Apart and Home/Work actions with consequences
    before activation. Retain a selected survivor until explicit replacement or
    dismissal. Make any retained roster an inspection route. Put advanced sort,
    relationship filters and batch assignment behind a labeled disclosure;
    replace P-H/P-W/ALL-H/ALL-W with readable scope and destination labels.
  - **Acceptance:** all displayed state comes from actual game data. Selection is
    predictable; a touch player can inspect before changing a role or relationship.
    Room capacity, compatibility, tension warnings and cleared-pin consequences
    remain visible beside the relevant action. No command requires hover prose.
  - **Verify:** normal/minimum sizes with six and ten survivors, long names,
    injured/away survivors, tense/supportive pairs, every roster page, full rooms
    and incompatible roles. Tap selection, change role, pair/separate, pin/clear
    and batch-copy paths, including Cancel and dismissal. Depends on UI-01/04.
  - **Current state:** survivor cards now select without mutating role; NEXT ROLE
    and PAIR / APART are explicit, and the inspector no longer invents energy,
    hunger, or health meters. Remaining work is live multi-page/long-name review
    and verification of every warning path.

- [ ] **UI-06 / P1 — Give action results temporary feedback and retain general event history.**
  - **Screen/files:** gameplay feedback and Log;
    `src/ui/toolbar_panel/log.rs::draw_log_context`,
    `src/state/game_state_{building_commands,mission_commands,priority_commands,log_commands,lifecycle}.rs`,
    `src/state/game_state_assign_space_commands.rs`, `src/data/event_log.rs`.
  - **Observed (code):** rejected missions, blocked assignments, undo/refunds,
    export results and autosave failures are pushed into `event_log`. Normal play
    has no general temporary-feedback renderer. Log displays only two recent
    general events before social history exists; once it exists, the social
    branch returns without displaying general events at all. A failed action can
    therefore appear inert and later lose its only accessible explanation.
  - **Change:** acknowledge actions near their affected object/control with brief
    text and appropriate world feedback. Coalesce routine events and reserve
    stronger emphasis for actionable failures; keep persistent hazards as
    current state, not never-expiring event messages. Add an accessible general
    event history alongside daily social reports, with full details and deliberate
    scrolling/paging. Preserve a quiet ongoing save-failure state until resolved,
    plus its retrievable error detail.
  - **Acceptance:** players can tell what changed or why an action was refused
    without opening a dashboard. Feedback expires, updated state remains correct,
    and important results can still be read after later events/day summaries.
    Save and export failures are understandable and not success-styled.
  - **Verify:** before and after the first daily report, trigger blocked placement,
    a mission prerequisite failure, incompatible pin, undo, successful action and
    controlled save/export failure. Observe feedback expiry and retrieve each
    result using touch at 720x480 and 1280x720. Depends on UI-01/04; supplies the
    history foundation for UI-07.
  - **Current state:** new event-log entries surface as a temporary, color-coded
    acknowledgement near the advisor banner, and Log now has a paged EVENT HISTORY
    view beside social reports. Live failure/recovery review remains.

- [ ] **UI-07 / P1 — Give social reports a readable, deliberate reading view.**
  - **Screen/files:** Log timeline, selected report, search and export;
    `src/ui/toolbar_panel/log.rs`, `src/ui/hit_zones/log.rs`,
    `src/state/game_state_log_commands.rs`, `src/ui/toolbar_panel/log_model.rs`.
  - **Observed (screenshot + code):** the game's central emergent stories are
    squeezed into three 13px rows below filters and repeated social metrics.
    Selected reports get a 320x68 floating box; detail and recommendation are
    single-line truncated strings. The saved Log screenshot visibly cuts the
    story mid-sentence. Export is as prominent as the reading controls.
  - **Change:** let Log become the dominant reading/comparison area while open.
    Use readable rows, a wrapped or scrollable full report and explicit Back/Close.
    Keep recommendation next to its story; move secondary metrics to disclosure
    and Export to a quiet archive utility area. Fit the search keyboard without
    covering its query, Done action or selected content. Preserve all existing
    filters, paging and export through understandable controls.
  - **Acceptance:** the player can read the entire story and recommendation
    without hover or exporting a file, then return to the same colony context.
    Search and report selection remain clear with long content and no results.
  - **Verify:** all baseline sizes; long titles/detail/recommendations, several
    days, empty archive and no-match query. Tap search, backspace, Done, Clear,
    filters, next/previous, full report, close and export. Depends on UI-01/04/06.
  - **Current state:** the Log now has readable timeline rows, wrapped report
    detail/recommendation text, a persistent CLOSE REPORT action, and fresh
    timeline/detail captures. Remaining work is keyboard/no-match/empty-archive
    capture and live touch verification.

- [ ] **UI-08 / P1 — Put mission readiness, risk and reward beside an explicit launch action.**
  - **Screen/files:** Research before/after an Exploration Gate and during missions;
    `src/ui/toolbar_panel.rs::draw_research_context`,
    `src/state/game_state_mission_commands.rs`,
    `src/systems/mission_system/planning.rs`, `src/data/technology.rs`.
  - **Observed (code only):** mission cards show name, minutes and a percentage
    without a risk label; description/rewards are hover details. Clicking launches
    directly, while no-gate/no-crew/cooldown reasons are only logged afterward.
    Technology progress uses the same button styling despite no corresponding
    action handler. The status line repeats unlocked counts and the next target.
  - **Change:** select a mission to reveal its crew availability, labeled danger,
    duration, reward profile and prerequisites beside a clear Launch control.
    Show the current blocking reason before activation. Render passive technology
    progress as state rather than a clickable button; disclose effects on tap.
    Before the gate exists, show its prerequisite and route to Build instead of
    the full advanced dashboard. Retain the scenario's technology requirement.
  - **Acceptance:** the player understands the mission's tradeoff before launch,
    sees why it is unavailable, and can tell an action from passive progress.
    New systems become visible when relevant without hiding survival constraints.
  - **Verify:** capture this previously unreviewed screen at 1280x720 and 720x480;
    tap through no gate, no available crew, cooldown, ready launch, active mission
    and unlocked technology. Check feedback/history after return. Depends on
    UI-01/04/06; do not claim present Research clipping without a capture.
  - **Current state:** Research now separates mission selection from action, shows
    risk/duration/reward/readiness, and routes a missing gate to Build. Fresh
    captures cover blocked and ready states; cooldown/no-crew and launch-result
    review remain to be exercised.

- [ ] **UI-09 / P2 — Consolidate placement information and make teaching contextual and reopenable.**
  - **Screen/files:** arrival, construction and help;
    `src/ui/introduction.rs`, `src/ui/toolbar_panel.rs::draw_build_context`,
    `src/ui/bottom_toolbar.rs`, `src/ui/gameplay/placement_preview.rs`,
    `src/state/game_state_building_commands.rs`, `assets/data/game_config.json`.
  - **Observed (screenshot + code):** placement repeats building/cost in the card,
    world label, large cursor panel and toolbar helper. The cursor panel obscures
    nearby world content. Generic construction prose stays in the tray. Arrival
    dismisses correctly, but no help-reopen route exists; its mission step says
    “MISSIONS” although the visible mode is Research. Assign help also names
    right-click despite the Filter Room touch path.
  - **Change:** keep one compact selected-plan summary next to the placement
    action: footprint, salvage cost/affordability, immediate effect and current
    blocking reason. Leave the world preview readable and keep Undo/Cancel close.
    Provide a touch preview-before-commit path where required to read warnings.
    Move general explanations to short first-use prompts and a visible Help
    disclosure, remove completed/generic instructions from ordinary play, and
    name actual controls consistently. Store revised player copy in JSON.
  - **Acceptance:** a player can see where the room goes, what it costs and why
    it is blocked without reading repeated panels. A touch-only new player can
    learn building, selection, room filtering and missions, dismiss teaching,
    and reopen it without restarting or replaying colony creation.
  - **Verify:** baseline sizes; four arrival steps, first placement, repeat
    placement, unaffordable/occupied/out-of-bounds placement, Undo, Cancel and
    Help reopen. Confirm full warnings remain available and no temporary hint
    becomes permanent wallpaper. Depends on UI-01/03/04/08.
  - **Current state:** placement feedback now keeps footprint, cost, effect, and
    blocking reason beside the preview, and touch copy uses the visible FILTER
    ROOM and Tap actions. The arrival mission step is named Research, and HELP
    reopens a concise decision guide without restarting the run. The full
    teaching review remains.

### Further inspection and implementation sign-off — not verified defects

- [ ] **UI-10 / P1 — Complete live visual and touch review of the supported screen flow.**
  - **Scope/files:** `scripts/capture_ui_smoke.ps1`, `src/main.rs`,
    `src/game.rs::begin_capture_scene`, `docs/verification/manual_relationship_playtest.md`,
    `README.md`, `game_page.json`, and the UI/input files changed above.
  - **Gap:** existing snapshots establish composition problems but not current
    browser behavior. Menu, Settings, all arrival steps, Research, Log keyboard,
    long/late-game data, urgent failures and victory/failure lack inspected live
    evidence. The smoke harness forces gameplay and mainly checks colored/nonblank
    regions; it cannot establish hierarchy or interaction correctness. The old
    manual checklist assumes permanent rails/meters that this plan removes.
  - **Work:** after each cohesive change, refresh only affected captures directly
    in `docs/verification/`, replacing equivalent images. Extend supported scene
    fixtures where needed and revise the manual checklist around player decisions.
    Inspect native and actual embedded browser canvas at 1280x720, 1920x1080 and
    the declared minimum (currently 720x480 baseline). Record actual canvas size
    and scaling; test narrower/portrait behavior to define support rather than
    asserting an untested phone minimum. Exercise touch input without relying on
    hover, keyboard or right-click, including pointer conversion after resize.
  - **Acceptance:** evidence records scene, size, input path and result for each
    applicable UI_STYLE §9 item, including 2–3 attention regions, full warnings,
    unclipped text, readable world targets and safe recovery. Clearly label any
    unsupported size, unavailable device or blocked test; screenshots alone do
    not count as a touch pass.
  - **Verify:** menu → arrival → build/cancel/undo → select/pin/filter → mission
    → daily report/search/detail/export → outcome/review/restart → menu/Continue.
    Include long names, large resource values, ten survivors, dense construction,
    injury, shortages and storage failure. In future implementation changes run
    focused regressions in `tests/` (target five cases per major feature), the
    source-size gate, and `./publish.ps1` without parameters; report results or
    blockers. Keep every Rust file within 800 physical lines. Do not run publish
    merely to validate this documentation-only audit.
