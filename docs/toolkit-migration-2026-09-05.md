# Toolkit migration

Tooltips now fit title/body text with toolkit measurement using the actual width
available inside the caller's bounds. Oversized tooltip geometry is constrained
to narrow bounds. The UI's character-count fitting wrapper is removed: toolbar,
assignment, advisor, objective, report, inspector, resource and placement text
all use shared measured truncation with their drawing font size and panel space.
Narrative excerpts stored in gameplay logs keep their domain character budgets.

Release captures exposed pre-existing row overflow in the right rail and portrait
overlap in the compact inspector. The right rail now uses the space beside the
centered toolbar, limits rows to its available height and points to Assign when
more colonists exist. Extra relationship portraits appear only when the inspector
has enough height to keep them below the health row.

SimulationRng already delegates its existing stream to `LegacyLcg64<1>`. The
sequence/zero-seed regression, bitwise float conversion and existing seeded tests
are retained. Game-owned range contracts remain unchanged. Data loading, sound,
effects, input, persistence and other shared facilities were audited against the
review; no additional reported duplication remains.

Validation: 159 tests including narrow tooltip geometry and legacy RNG fixtures,
formatting, strict all-feature Clippy, default Windows/WebGL Preview publisher
and tracking, and native release UI captures. The long-tooltip capture explicitly
exercises 180-pixel and 320-pixel bounds with wide glyphs and long descriptions.
