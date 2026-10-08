# The Final Landing — Campaign Pacing Report

Headless campaigns played by the scripted colony AI (`src/autoplay.rs`) through the same commands as the player. Estimated hours assume the documented speed mix of 5.2 real minutes per in-game day, before pauses.

| Site | Difficulty | Seed | Act start days | Final day | Est. hours | Outcome | Pop (peak) | Deaths | Births | Techs | Relics | Expeditions | Friendships | Mood |
| --- | --- | ---: | --- | ---: | ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| verdant_basin | standard | 11 | 1 / 23 / 61 / 123 / 199 | 279 | 24.2 | Victory (rootbound) | 35 (35) | 0 | 3 | 62 | 117 | 71 | 26 | 60 |
| frost_shelf | standard | 23 | 1 / 24 / 61 / 122 / 213 | 326 | 28.3 | Victory (ascendant) | 21 (21) | 2 | 3 | 65 | 125 | 79 | 14 | 65 |
| ashen_steppe | standard | 37 | 1 / 15 / 61 / 95 / 192 | 307 | 26.6 | Victory (ascendant) | 29 (29) | 0 | 2 | 67 | 121 | 76 | 22 | 60 |
| verdant_basin | gentle | 41 | 1 / 14 / 61 / 103 / 140 | 208 | 18.0 | Victory (rootbound) | 15 (15) | 1 | 0 | 57 | 85 | 44 | 11 | 55 |
| verdant_basin | harsh | 53 | 1 / 80 / 87 / 157 / 246 | 283 | 24.5 | Victory (rootbound) | 22 (22) | 0 | 0 | 52 | 93 | 42 | 12 | 52 |

## Discovery

Each landing draws its own native species; the colony finds them in an order set by where their territories fall. Inspired technologies are revealed by gathering those species.

| Site | Seed | Native species | Order found | Inspired techs revealed / researched |
| --- | ---: | --- | --- | ---: |
| verdant_basin | 11 | Spinegrass, Glassvine, Limestone, Basalt, Rust Iron, Singing Quartz, Shellbacks, Sweetpods, Ember Tubers, Glyph Stones, Crystal Archive, Dormant Engine, Silkreed, Barkweave | Shellbacks → Basalt → Spinegrass → Singing Quartz → Sweetpods → Crystal Archive → Glyph Stones → Dormant Engine → Ember Tubers → Limestone → Glassvine → Silkreed → Barkweave | 19 / 19 |
| frost_shelf | 23 | Spinegrass, Glassvine, Glassrock, Sunstone, Singing Quartz, Cobalt Glass, Glowfruit, Ember Tubers, Sweetpods, Crystal Archive, Glyph Stones, Barkweave, Shellbacks, Dormant Engine | Sunstone → Sweetpods → Spinegrass → Singing Quartz → Glassrock → Ember Tubers → Barkweave → Glyph Stones → Crystal Archive → Shellbacks → Dormant Engine → Glowfruit → Cobalt Glass → Glassvine | 21 / 21 |
| ashen_steppe | 37 | Silkreed, Spinegrass, Sunstone, Basalt, Rust Iron, Verdigris Copper, Bitterleaf, Shellbacks, Sweetpods, Dormant Engine, Crystal Archive, Glassvine, Glyph Stones, Veilcaps | Shellbacks → Rust Iron → Silkreed → Bitterleaf → Verdigris Copper → Basalt → Dormant Engine → Glassvine → Crystal Archive → Glyph Stones → Veilcaps → Sunstone | 21 / 21 |
| verdant_basin | 41 | Glassvine, Silkreed, Glassrock, Basalt, Cobalt Glass, Verdigris Copper, Veilcaps, Ember Tubers, Reedgrain, Crystal Archive, Dormant Engine, Sunstone, Glyph Stones, Sweetpods | Glassrock → Glassvine → Veilcaps → Verdigris Copper → Cobalt Glass → Basalt → Ember Tubers → Reedgrain → Crystal Archive → Sunstone → Glyph Stones → Dormant Engine → Sweetpods → Silkreed | 22 / 22 |
| verdant_basin | 53 | Silkreed, Spinegrass, Basalt, Sunstone, Cobalt Glass, Rust Iron, Reedgrain, Veilcaps, Shellbacks, Dormant Engine, Glyph Stones, Barkweave, Bitterleaf, Glassvine | Veilcaps → Sunstone → Silkreed → Shellbacks → Rust Iron → Cobalt Glass → Basalt → Spinegrass → Reedgrain → Barkweave → Bitterleaf → Glassvine | 18 / 18 |
