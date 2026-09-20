# Manual Relationship Playtest Checklist

Use this checklist after relationship, Assign-mode, Log-mode, or character-visual changes. Automated tests and smoke captures cover regressions; this pass checks whether the relationship manager is readable and playable.

## Setup

- Run `cargo test`.
- Run `.\scripts\capture_ui_smoke.ps1`.
- Start a fresh game from the main menu at 1280x720 or larger.
- Repeat the readable-control checks at 720x480 with taps instead of hover/right-click actions.
- Keep time paused while checking UI affordances, then run at normal speed for daily-routine checks.

## First Frame Readability

- Confirm the colony occupies the main field, with the compact status strip showing the urgent alert and essential food/salvage/mood values.
- Open Colony and confirm the four active objectives replace the world area without a permanent rail.
- Open Assign and confirm survivor cards expose actual role, room-pin, activity, mood, and relationship state without invented need bars.
- Confirm support/tension markers and body-language frames are visible on the map before opening Log mode.
- Complete the four-step arrival briefing with Continue / Enter Colony and confirm the active colony opens with the configured roster.

## Assign Mode Decisions

- Open Assign mode and confirm the selected survivor stays pinned at the start of the roster.
- Cycle filters through ALL, RISK, TENSE, PLUS, PIN, and at least one role filter; confirm visible rows match the selected filter.
- Cycle sorting through ORD, MOOD, BOND, and R-ALL; confirm low-mood or high-pressure survivors move as expected.
- Tap survivor cards to select and inspect without changing a role; use NEXT ROLE for an explicit role change.
- Arm PAIR / APART before selecting a second survivor, then confirm the directive and warning text update.
- Click a compatible Habitat and work room on the map for the selected survivor; confirm HOME/WORK map labels and warnings update.
- Tap Filter Room, then tap a room or work space, and confirm the roster filters to survivors pinned to that specific building instance. Right-click remains an optional desktop shortcut.
- Use the labeled page/colony home/work copy controls; confirm capacity and compatibility warnings prevent bad silent assignments.

## Relationship Pressure Loop

- Put a tense pair in the same Habitat or same compatible work room and confirm warning text appears.
- Put a supportive pair together and confirm the assignment reads as beneficial or neutral.
- Advance time through work/eating/recovery periods and confirm daily routine contact changes relationship pressure.
- Confirm strong support/tension creates pulsing social markers and alternates support/tension body-language frames.
- Change colony priority to Recovery, Stockpile, and Survey from the Colony tray; confirm the relationship loop still remains readable.

## Log And Archive

- Advance through at least three daily summaries.
- Open Log mode and confirm the social brief names mood, close/tense pair counts, and strongest signal while the colony remains visible above it.
- Filter the social archive by ALL, TENSE, and PLUS; confirm only matching reports remain.
- Search by survivor name, relationship wording, recommendation text, and day number.
- Open a report drilldown and confirm the full story plus recommendation are visible without hover or layout overflow, then use CLOSE REPORT.
- Use the touch keyboard to enter a search, then export the archive and confirm `docs\exports\social_archive.md` contains newest reports first on Windows. In a browser, confirm the Markdown is copied through the shared clipboard bridge; if clipboard access is blocked, use its visible copy/download fallback.

## End-To-End Outcome

- Select mission cards in Research, confirm risk/duration/reward/readiness, then use the explicit LAUNCH or BUILD GATE action.
- Reach Day 7 victory or trigger a failure case.
- Confirm the final outcome is understandable from objectives, alerts, resources, and social history.
- Restart, then use Continue from the menu and confirm Assign/Log state does not leak from the previous run.
