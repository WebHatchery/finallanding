# The Final Landing — Game Design (Rebuild)

This document replaces the one-year MVP brief (`tfl_mvp.md`). It records the
review of the original design, the reasons the first execution failed, and the
design of the rebuilt game: a colony simulation about relationships, a dynamic
technology tree, and autonomous survivors designed with the Prometheus
agent-oriented methodology.

## 1. Review of the original design

The original brief had one pillar — **indirect control through space**:
buildings + proximity + time = relationships. The player is the colony's AI.
It never commands a survivor; it shapes where people live, work, eat and rest,
then reads the story it accidentally wrote in the daily log.

That pillar is kept. What it lacked was a world worth inhabiting for long:

| Original choice | Problem found in the first build | Rebuild decision |
| --- | --- | --- |
| 6–10 colonists with one trait | Survivors felt interchangeable; behaviour came from scripted schedules | Generated survivors with several traits, eight skills, passions, an ambition, memories and BDI reasoning |
| No tech tree | Nothing to plan beyond the first day | A **dynamic** tech tree grown from the native species each colony gathers, discovered through colonist activity |
| 5 buildings, 2 resources | No production chains, no reason to rearrange a settlement | ~30 buildings, 9 stockpiled resources, power, farming, crafting, medicine and defence |
| Day 7 victory (≈34 minutes) | The run ended before relationships could matter | A five-act campaign of five in-game years (≈300 days, 30–40 hours) |
| Abstract jobs assigned by the player | Colonists were puppets with a role label | Survivors choose their own goals; the player sets policies, priorities and spaces |
| Isometric crash-site art at 1280×720 | Wreckage dominated the map, survivors clumped into one unreadable stack, controls overlapped | A clean top-down map composed for a fixed **1920×1080** virtual canvas |
| Permanent rails and dashboards | Several panels competed with the colony | One dominant world view; information appears on selection or in overlays |

## 2. Experience goals

- **Watch minds, not meters.** Selecting a survivor shows what they believe,
  which goal they are pursuing, the plan they chose, and why they switched.
- **Indirect influence.** Build, zone priorities, research, policies and
  expedition calls. Survivors may refuse, negotiate, or find another way.
- **Relationships with consequences.** Friendships speed shared work,
  rivalries spill into refusals and fights, partnerships produce families,
  and grief can break a colony.
- **A long arc.** Each run is a 30–40 hour campaign with an ending chosen by
  the colony, not only by the player.
- **Hundreds of hours of replay.** Seeded maps, three landing sites, three
  endings, three difficulties, generated crews, a reshaped tech tree and an
  event deck that reacts to the colony.

## 3. Campaign structure

A year has four seasons of fifteen days (sixty days a year). The campaign
spans five years. Acts advance on objectives, not on the calendar, but each
act has a climactic event that tests what the colony built.

| Act | Typical days | Player decision | Climax |
| --- | --- | --- | --- |
| I · Landfall | 1–30 | Shelter, food, power and the first research | First storm front |
| II · Roots | 30–100 | Farms, workshops, first expeditions, first winter | The Long Frost |
| III · Echoes | 100–180 | Decode the alien signal: relics, xenology, signal mast | Fauna migration and blight |
| IV · Divergence | 180–250 | The colony votes on its future: Beacon, Rootbound or Ascendant | Dissent from those who disagree |
| V · The Final Landing | 250–300 | Build the chosen megaproject and endure the last crisis | Ending and epilogue |

**Endings.** *Beacon* calls the fleet home with a repaired long-range array.
*Rootbound* binds the colony to the planet's living network. *Ascendant*
awakens the precursor spire. Each needs its own capstone technology and
megaproject, and survivors hold their own views on which future is right.
The colony **fails** if every survivor dies or leaves, or if colony morale
stays in collapse for too long.

### Pacing target

At 1× speed one day lasts 8 real minutes, 4 at 2×, 2 at 3×. A typical mix
(40% at 1×, 40% at 2×, 20% at 3×) averages ≈5.2 minutes a day, so 300 days
take ≈26 hours of simulated time; pausing to plan, read the chronicle and
inspect minds brings a run to 30–40 hours. The headless campaign runner
(`TFL_CAMPAIGN_REPORT_PATH`) verifies that a scripted colony reaches an ending
in the intended day band.

### Verified pacing

`docs/verification/campaign_report.md` records the scripted colony AI playing
every landing site and difficulty. All runs reach a victory; Standard runs end
between days 279 and 326 (≈24–28 hours at the speed mix above, before
pauses), Gentle around day 210 and Harsh near day 285. Act IV opens between
days 95 and 157 depending on the site. The report's Discovery table shows
each run's native species and the order they were found. A human player who
pauses to plan and read will take longer than the script, which never stops.

### Replayability

- **Landing sites:** Verdant Basin (temperate), Frost Shelf (long winters,
  rich ore), Ashen Steppe (dry, fertile ash, frequent storms).
- **Difficulty:** Gentle, Standard, Harsh — need decay, event pressure and
  starting stores.
- **Seeds:** map layout, native species and their territories, ruins, crew,
  tech-tree shape and event deck all derive from one run seed.
- **Generated survivors:** name, age, traits, skills, passions, ambition and
  backstory. No two crews reason alike.
- **Dynamic tech tree:** technologies are inspired by the native species each
  colony gathers, in the order it finds them; prerequisites, hidden
  discoveries, forks and costs also change every run (§6).
- **Three endings** with different capstones and political consequences.
- **Legacy:** achievements persist between runs.

## 4. Prometheus agent design

The survivors are designed with the Prometheus methodology (Padgham &
Winikoff): system specification, architectural design, then detailed design,
on a Belief–Desire–Intention (BDI) execution model.

### 4.1 System specification

**System goals.** Keep every survivor alive; keep the colony fed, sheltered,
warm and powered; grow knowledge; nurture relationships; fulfil personal
ambitions; reach a campaign ending.

**Percepts** (what a survivor can sense — always local):

| Percept | Source | Range |
| --- | --- | --- |
| Nearby survivors, their activity and visible mood | World | Sight radius (shrinks at night and in storms) |
| Resource nodes and their remaining yield | World | Sight radius |
| Loose items, blueprints, damaged structures | World | Sight radius |
| Creatures (threats) | World | Sight radius |
| Weather, time of day, temperature | World | Global |
| Own needs, health and inventory | Body | Self |
| Messages (inform, request, warn, chat, proposal) | Other agents | Delivered to inbox |
| Colony policies and priorities | Colony comms | Global |

**Actions:** move along a path, work at a target, pick up, deposit, consume,
sleep, talk, send a message, flee, fight, wait.

**Key scenarios.** *Hungry with an empty mess hall:* the survivor tries the
mess hall, learns it is empty, takes raw stores, forages a known wild-food patch
bush, and finally asks a friend for food. *Construction interrupted by a
creature:* the builder flees indoors, warns others, and resumes the site
afterwards. *A rival on the same shift:* a survivor with low opinion of a
co-worker chooses a different job or confronts them.

### 4.2 The six agent properties

| Property | Mechanism |
| --- | --- |
| **Situated** | Each survivor only knows what it perceived or was told. Beliefs about resource nodes, threats and people come from percepts inside a sight radius, and they decay. The map's fog of war is the union of what survivors have seen. |
| **Autonomous** | Goals are generated from the survivor's own needs, traits, skills, passions and ambition. The player changes the environment (buildings, policies, priorities), never the survivor's intention. |
| **Reactive** | Interrupting percepts (threat sighted, need critical, injury, storm, urgent message) trigger immediate re-deliberation and can pre-empt the current intention. |
| **Proactive** | Goals persist. A committed intention keeps a commitment bonus, so the survivor pursues it across interruptions instead of dithering; ambitions persist for years. |
| **Flexible** | Every goal has several plans with context conditions (for example *Eat*: mess-hall meal, raw stores, forage, ask a friend). The best applicable plan is chosen by preference and past outcomes. |
| **Robust** | When a step fails the plan fails, beliefs are corrected (the bush is bare, the path is blocked), the failed plan is excluded, and the next applicable plan is tried. Exhausted goals back off and may become a help request. |
| **Social** | Survivors exchange typed messages: inform (share a belief), request help, accept/decline, warn, chat, comfort, insult, propose. Relationships change through interaction, gossip propagates opinions, and partners form families. |

### 4.3 Architectural design

One agent type, **Survivor**, plays several roles chosen at run time:
Provider (food), Builder, Hauler, Gatherer, Researcher, Medic, Crafter,
Guardian, Explorer and Companion. Children play only Companion and Learner.
**Creatures** are simple reactive agents (forage, raid, flee) used as threats.

Interaction protocols:

- **Help request:** `RequestHelp(goal, place)` → recipients weigh opinion,
  their own urgency and kindness → `Accept` (adopt a *Help* goal) or
  `Decline`. The requester waits, then falls back to another plan.
- **Information sharing:** during a chat a survivor sends `Inform(fact)`;
  the receiver merges the fact into its beliefs if it trusts the sender.
- **Warning:** a survivor seeing a threat broadcasts `Warn(threat)` to
  survivors in earshot, who re-deliberate (flee or defend).
- **Courtship:** `Propose` → acceptance depends on mutual opinion and romance.

### 4.4 Detailed design

- **Beliefs** (`agents/beliefs.rs`): known nodes, threats, failed places,
  opinions of other survivors, last-known stockpile view.
- **Goals** (`agents/goals.rs`): survival (eat, sleep, warmth, flee, heal),
  work (build, haul, farm, gather, cook, craft, research, treat, guard),
  social (chat, comfort, court, reconcile), help, ambition, recreation and
  mental-break goals. Each goal is scored by a utility function.
- **Plans** (`agents/plans.rs`, `agents/plan_library.rs`): ordered steps with a
  context condition and preference. Multiple plans per goal.
- **Intentions** (`agents/intention.rs`): the committed goal, chosen plan,
  step cursor, failed-plan list and commitment.
- **Deliberation** (`agents/deliberation.rs`): periodic or interrupt-driven
  goal selection with hysteresis.
- **Messages** (`agents/messages.rs`): inbox-based, delivered next tick.

## 5. Survivors

- **Needs:** food, rest, warmth, social, recreation, safety (0–100).
- **Mood:** baseline + need pressure + thoughts (memories with value and
  decay). Low mood causes mental breaks: wandering, sulking, binge eating,
  lashing out.
- **Health:** injuries and illness; infirmaries and medicine heal; death
  produces grief in everyone who cared.
- **Skills:** Construction, Farming, Cooking, Science, Medicine, Crafting,
  Exploration, Social — level 0–20, passions speed learning and lift mood.
- **Traits** (two or three each): Social, Loner, Brave, Cautious, Kind,
  Abrasive, Optimist, Pessimist, Diligent, Lazy, Curious, Romantic,
  Stubborn, Gourmand, Green Thumb, Tinkerer, Spiritual, Leader, Hot-headed,
  Night Owl.
- **Ambitions:** master a skill, raise a family, befriend the colony, map the
  wilds, decode the spire, build a home, lead the council.
- **Life cycle:** partners may have children (with housing and policy);
  children grow into adults over two years.

## 6. Dynamic technology tree

The tree grows from what each colony finds. The data files list 79
technologies in six branches — Survival, Agronomy, Industry, Medicine,
Society, Xenology — across five tiers, plus three capstones, and 22 native
species (`assets/data/finds.json`).

**Native species.** Every landing draws its own species: three foods, two
fibres, two stones, two ores and two kinds of ruin, out of seven, four, four,
four and three. Each species is given territories on the map (beside water,
rock, soil or sand when it prefers them), so which species lie closest to the
crash site — and so which the crew gathers first — changes every run. Food
species differ in yield, and plantable ones in how much an orchard grows and
how fast: Ember Tubers are heavy and slow, Veilcaps thin and quick, Shellbacks
rich but cannot be farmed.

**Inspiration.** Most technologies are *inspired*: they name the species that
suggest them and how much the colony must gather. Until then they are not
shown at all. Gathering a species for the first time is logged ("Rosa brings
back the colony's first Reedgrain"), and crossing a technology's threshold
reveals it ("Reedgrain inspires a new line of study: Reedgrain Milling").
Each species has its own technology (Tuber Beds, Spore Tinctures, Basalt
Footings, Cobalt Lenses, Engine Autopsy…), and core technologies take any
species of a kind (Smelting needs any ore, Native Cultivation any plantable
food). A technology inspired only by species this landing lacks is **absent**
for the run. Expeditions can carry home a sample of a foreign species
(at most three a run), which brings its technologies within reach.

**Guarantees.** Validation requires every technology that unlocks a
building, recipe or policy to list enough species of a kind that any draw
includes one, and inspired technologies are never drawn as prerequisites, so
no run can lock itself out of content. Tests cover 24 seeds on every site.

**The crew's own knowledge** (shelters, field medicine, power, labs, the
capstone paths) is regenerated from the seed as before:

- Prerequisites are drawn from earlier tiers, favouring the same branch and
  sometimes crossing branches.
- About a third of it starts **hidden** and is revealed by *insight*:
  survivors practising related work accumulate branch insight and occasionally
  have a eureka. Relics and expeditions reveal others.
- **Forks** are mutually exclusive pairs; researching one locks the other.
- **Cost adapts**: a branch the colony practises becomes cheaper.
- Researchers work at labs. If the player sets no focus, curious researchers
  pick a project from their own interests.

**Native orchards** grow whichever plantable food the colony has gathered
most, with that species' yield and growing time, so a colony's farming
follows its foraging.

The research overlay draws only known technologies, packed by branch and
tier, so the tree visibly takes a different shape each run. A strip of
native finds lists the species in the order they were found with what has
been gathered; inspired cards carry their species' colour, and the map draws
each resource node as its own species (tubers in a mound, fungus caps,
shells, hexagonal basalt, crystal ores, glyph stones, the dormant engine) in
that species' colour.

## 7. Colony systems

- **Resources:** Food, Meals, Salvage, Fibre, Stone, Metal, Components,
  Medicine, Relics; power is a flow (generation versus demand).
- **Buildings** are blueprints delivered and built by survivors. Categories:
  Shelter, Food, Industry, Science, Care, Power, Social, Defence, Expedition
  and Capstone.
- **Weather and seasons:** temperature, storms, cold snaps, drought, blight.
- **Threats:** native fauna raid crops and stores; brave survivors defend,
  cautious ones flee and warn.
- **Expeditions:** pick a discovered site; survivors volunteer according to
  bravery, curiosity and ambition, and each call the colony could not fill
  weighs on their sense of duty (more for the diligent) until someone goes;
  they return with loot, relics, technologies, samples of foreign species,
  recruits, injuries or stories.
- **Policies:** rationing, work hours, curfew, family planning, expedition
  stance. Survivors judge policies through their traits.
- **Chronicle:** the colony AI writes an end-of-day summary and keeps a
  searchable history of events, relationships and discoveries.

## 8. Screen briefs (1920×1080 normal, 1280×720 minimum)

The game draws into a fixed 1920×1080 virtual canvas, letterboxed to the
window. 1920×1080 is the normal size; 1280×720 is the minimum supported
window (scale 0.67). Touch targets are at least 48 virtual pixels.

| Phase | Current decision | Dominant focus | Primary action | Supporting information | Deferred information |
| --- | --- | --- | --- | --- | --- |
| Title | Start, continue or configure a run | Crash-site art and the menu | New Colony / Continue | Save summary | Settings, achievements |
| New colony | Choose site, difficulty and seed | Site cards | Land | Site description and modifiers | Crew details until landing |
| Observation | Which pressure to address next | The colony map | Open a tool or select something | Resources, date, act objectives, active alerts | Roster, tech tree, relations, chronicle |
| Placement | Does this building fit here? | Ghost footprint on the map | Place / Cancel | Cost, requirements and blocking reason | Other categories |
| Inspection | Understand a survivor or building | Inspector panel beside the map | Close or follow | Mind: goal, plan, beliefs; needs; relations | Full history in chronicle |
| Research | Which technology next | Tech tree overlay | Set focus | Cost, unlocks, prerequisites, forks, inspiring species, native finds | Hidden and absent technologies |
| Expeditions | Where to send people | Site list | Call for volunteers | Danger, duration, rewards | Results until return |
| Relations | Who is close or in conflict | Relationship web | Select a survivor | Opinion values and status | Thought history |
| Chronicle | What happened and why | Daily entry list | Read and filter | Category filters | Older years via paging |
| Divergence vote | Which future to propose | Three ending cards | Propose | Projected support per survivor | Capstone details |
| Results | Review the ending | Epilogue | New Colony / Menu | Per-survivor fates | — |

Layout at 1920×1080: a 56 px top bar (date, resources, speed, menu) and a
76 px bottom toolbar frame the world. The act tracker and alerts sit in the
upper-left corner over the map; the inspector slides in on the right (460 px)
only while something is selected. Research, Colonists, Relations, Expeditions,
Policies and Chronicle open as overlays that replace the map focus.
