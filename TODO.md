# TODO — The Final Landing

## Playtest feedback

- Enlarge the crash-site map; the current playable area reads as far too small for the colony it holds.
- Improve map art readability so terrain, wreckage, buildings, and survivors are distinguishable at a glance.
- Keep the colonist roster fully on screen; the list currently runs past the bottom edge.
- Revisit new-game setup so a run does not open with an already-populated colonist list.
- Give the main menu Continue, Settings, and Exit alongside Start Game.
- Draw the build tooltip above the build menu instead of behind it.

## Production art

- Replace the generated 128px survivor portraits and pose frames with higher-fidelity production art.
- Replace the procedural crash-site dressing and room interiors with an authored isometric tile and building set.
- Add richer relationship animation frames once final character art exists.

## Engineering

- Split runtime colony state from persistence DTOs so relationship, assignment, and log changes can migrate cleanly.
- Profile and cache relationship-log rendering and daily report generation for the right rail and Log mode.
- Keep `scripts/capture_ui_smoke.ps1` captures and the manual relationship checklist current as UI layout changes land.
