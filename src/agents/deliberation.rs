//! Deliberation: generating candidate goals and scoring their utility.
//!
//! Goals come from the survivor's own needs, personality, skills, ambition and
//! relationships, read against what they believe and what the colony has
//! posted. Nothing here is an order from the player.

use super::beliefs::Bond;
use super::generation::{ambition_kind, mastery_skill};
use super::goals::{BreakKind, Goal, HelpNeed, WorkType};
use super::{Activity, Agent};
use crate::colony::Colony;
use crate::data::people::AmbitionKind;
use crate::data::{game_data, Need, NodeKind, Skill};
use crate::world::{AgentId, Calendar, StructureId, Tile, World};

/// What one survivor can see of another without reading their mind.
#[derive(Clone, Debug)]
pub struct AgentSnapshot {
    pub id: AgentId,
    pub name: String,
    pub tile: Tile,
    pub present: bool,
    pub mood: f32,
    pub activity: Activity,
    pub is_child: bool,
    pub partner: Option<AgentId>,
    pub needs_care: bool,
    pub goal: Option<Goal>,
    pub inside: Option<StructureId>,
}

impl AgentSnapshot {
    pub fn of(agent: &Agent) -> Self {
        Self {
            id: agent.id,
            name: agent.given_name.clone(),
            tile: agent.tile,
            present: agent.is_present(),
            mood: agent.mood,
            activity: agent.activity,
            is_child: agent.is_child,
            partner: agent.partner,
            needs_care: agent.health.needs_care(),
            goal: agent.current_goal(),
            inside: agent.inside,
        }
    }

    pub fn is_available_to_talk(&self) -> bool {
        self.present
            && !matches!(
                self.activity,
                Activity::Sleeping | Activity::Fleeing | Activity::Fighting | Activity::Breaking
            )
    }
}

/// Read-only view of everything a survivor's deliberation may consult.
pub struct View<'a> {
    pub world: &'a World,
    pub colony: &'a Colony,
    pub others: &'a [AgentSnapshot],
    pub calendar: Calendar,
}

impl View<'_> {
    pub fn snapshot(&self, id: AgentId) -> Option<&AgentSnapshot> {
        self.others.iter().find(|s| s.id == id)
    }

    /// How many other survivors already pursue this goal.
    pub fn pursuing(&self, goal: &Goal, me: AgentId) -> usize {
        let key = goal.key();
        self.others
            .iter()
            .filter(|s| s.id != me && s.goal.is_some_and(|g| g.key() == key))
            .count()
    }

    pub fn is_work_hours(&self, agent: &Agent) -> bool {
        let time = &game_data().balance.time;
        let shift = self.colony.work_hours_shift();
        let owl = if agent.personality.night_owl {
            2.0
        } else {
            0.0
        };
        let hour = self.calendar.hour();
        hour >= time.work_start_hour + owl && hour < time.work_end_hour + shift + owl
    }

    pub fn is_sleep_time(&self, agent: &Agent) -> bool {
        let time = &game_data().balance.time;
        let owl = if agent.personality.night_owl {
            3.0
        } else {
            0.0
        };
        let hour = (self.calendar.hour() - owl).rem_euclid(24.0);
        hour >= time.sleep_hour || hour < time.dawn_hour
    }
}

pub struct Candidate {
    pub goal: Goal,
    pub utility: f32,
    pub reason: String,
}

fn deficit(value: f32) -> f32 {
    ((100.0 - value) / 100.0).clamp(0.0, 1.0)
}

fn reason(key: &str, value: f32) -> String {
    format!("{} {:.0}", game_data().label(key), value)
}

fn survival_candidates(agent: &Agent, view: &View, out: &mut Vec<Candidate>) {
    let urgent = game_data().balance.needs.urgent_threshold;
    let food = agent.needs.get(Need::Food);
    let hour = view.calendar.hour();
    let mealtime = [7.5, 12.5, 18.5].iter().any(|t| (hour - t).abs() < 1.0);
    let mut eat = deficit(food).powi(2) * 1.5;
    if food < urgent {
        eat += 0.7;
    }
    if mealtime && food < 70.0 {
        eat += 0.18;
    }
    out.push(Candidate {
        goal: Goal::Eat,
        utility: eat,
        reason: reason("reason_hunger", food),
    });

    let rest = agent.needs.get(Need::Rest);
    let mut sleep = if view.is_sleep_time(agent) && rest < 92.0 {
        0.62 + deficit(rest) * 0.6
    } else {
        deficit(rest).powi(2) * 1.3 + if rest < 15.0 { 0.8 } else { 0.0 }
    };
    if view.colony.curfew() && view.is_sleep_time(agent) {
        sleep += 0.2;
    }
    // Nobody lies down beside a known predator unless they are collapsing.
    let threatened = agent
        .beliefs
        .nearest_threat(agent.tile)
        .is_some_and(|(_, distance)| distance < 7.0);
    if threatened && rest > 5.0 {
        sleep *= 0.25;
    }
    out.push(Candidate {
        goal: Goal::Sleep,
        utility: sleep,
        reason: reason("reason_tired", rest),
    });

    let warmth = agent.needs.get(Need::Warmth);
    if warmth < 60.0 {
        let utility = deficit(warmth).powi(2) * 1.6 + if warmth < 20.0 { 0.6 } else { 0.0 };
        out.push(Candidate {
            goal: Goal::WarmUp,
            utility,
            reason: reason("reason_cold", warmth),
        });
    }

    if let Some((threat, distance)) = agent.beliefs.nearest_threat(agent.tile) {
        let fearful = threat.dangerous || agent.personality.bravery < -0.2 || agent.is_child;
        if distance < 7.0 && fearful {
            let courage = agent.personality.bravery.clamp(-0.5, 0.8);
            let utility = 1.4 * (1.0 - courage * 0.6) * (1.0 - distance / 10.0).max(0.3);
            out.push(Candidate {
                goal: Goal::Flee,
                utility,
                reason: reason("reason_threat", distance),
            });
        }
    }

    if agent.health.needs_care() {
        let ill = if agent.health.ill { 0.45 } else { 0.0 };
        let utility = (1.0 - agent.health.hp / 100.0) * 1.3 + ill;
        out.push(Candidate {
            goal: Goal::SeekCare,
            utility,
            reason: reason("reason_health", agent.health.hp),
        });
    }

    let recreation = agent.needs.get(Need::Recreation);
    if recreation < 65.0 {
        let hours = if view.is_work_hours(agent) { 0.6 } else { 1.4 };
        let utility = deficit(recreation).powi(2) * 0.9 * hours;
        out.push(Candidate {
            goal: Goal::Relax,
            utility,
            reason: reason("reason_bored", recreation),
        });
    }
}

/// The best person to talk to: liked, familiar, nearby and awake.
fn chat_partner(agent: &Agent, view: &View) -> Option<(AgentId, f32)> {
    view.others
        .iter()
        .filter(|s| s.id != agent.id && s.is_available_to_talk())
        .filter(|s| s.tile.distance(agent.tile) < 30.0)
        .map(|s| {
            let relation = agent.beliefs.relation(s.id);
            let opinion = relation.map(|r| r.opinion).unwrap_or(0.0);
            let familiarity = relation.map(|r| r.familiarity).unwrap_or(0.0);
            let closeness = 1.0 - s.tile.distance(agent.tile) / 30.0;
            let score = opinion * 0.02
                + familiarity * 0.005
                + closeness
                + agent.personality.sociability * 0.2;
            (s.id, score)
        })
        .max_by(|a, b| a.1.total_cmp(&b.1))
}

fn social_candidates(agent: &Agent, view: &View, out: &mut Vec<Candidate>) {
    let social = agent.needs.get(Need::Social);
    let off_hours = if view.is_work_hours(agent) { 0.7 } else { 1.4 };
    if social < 75.0 {
        if let Some((partner, score)) = chat_partner(agent, view) {
            let utility = deficit(social).powf(1.5)
                * 0.85
                * (1.0 + agent.personality.sociability * 0.6)
                * off_hours
                + score.max(0.0) * 0.05;
            out.push(Candidate {
                goal: Goal::Chat(partner),
                utility,
                reason: reason("reason_lonely", social),
            });
        }
    }
    let seeks_family = ambition_kind(agent) == Some(AmbitionKind::RaiseFamily);
    let smitten = agent.beliefs.relations.values().any(|r| r.opinion > 55.0);
    if !agent.is_child
        && agent.partner.is_none()
        && (agent.personality.romance > 0.0 || seeks_family || smitten)
    {
        let crush = agent
            .beliefs
            .relations
            .iter()
            .filter(|(id, r)| {
                r.romance > 15.0
                    || r.opinion > 45.0
                        && view
                            .snapshot(**id)
                            .is_some_and(|s| !s.is_child && s.partner.is_none() && s.present)
            })
            .max_by(|a, b| (a.1.romance + a.1.opinion).total_cmp(&(b.1.romance + b.1.opinion)));
        if let Some((id, relation)) = crush {
            let utility =
                0.3 * (1.0 + agent.personality.romance) * off_hours + relation.romance * 0.004;
            out.push(Candidate {
                goal: Goal::Court(*id),
                utility,
                reason: reason("reason_romance", relation.romance),
            });
        }
    }
    if agent.personality.kindness > 0.0 {
        let hurting = view.others.iter().find(|s| {
            s.id != agent.id
                && s.present
                && s.mood < 30.0
                && agent.beliefs.opinion_of(s.id) > 20.0
                && s.tile.distance(agent.tile) < 20.0
        });
        if let Some(friend) = hurting {
            let utility = 0.55 * (0.5 + agent.personality.kindness);
            out.push(Candidate {
                goal: Goal::Comfort(friend.id),
                utility,
                reason: reason("reason_friend_low", friend.mood),
            });
        }
    }
    if agent.personality.kindness > 0.0 || agent.personality.leadership > 0.0 {
        let rival = agent.beliefs.relations.iter().find(|(id, r)| {
            r.bond() == Bond::Rival
                && view
                    .snapshot(**id)
                    .is_some_and(|s| s.is_available_to_talk())
        });
        if let Some((id, relation)) = rival {
            out.push(Candidate {
                goal: Goal::Reconcile(*id),
                utility: 0.2,
                reason: reason("reason_rivalry", relation.opinion),
            });
        }
    }
}

fn help_candidates(agent: &Agent, view: &View, out: &mut Vec<Candidate>) {
    if let Some((requester, need)) = agent.help_task {
        if view.snapshot(requester).is_some_and(|s| s.present) {
            let utility = match need {
                HelpNeed::Food | HelpNeed::Care => 1.0,
                HelpNeed::Defence(_) => 0.9 * (0.5 + agent.personality.bravery),
            };
            out.push(Candidate {
                goal: Goal::Help { requester, need },
                utility,
                reason: game_data().label("reason_asked").to_owned(),
            });
        }
    }
    let call = &view.colony.expeditions.call;
    if let Some(call) = call {
        if !agent.volunteered && !agent.is_child && agent.health.hp > 60.0 && agent.mood > 30.0 {
            let mut willingness = 0.2
                + agent.personality.bravery * 0.3
                + agent.personality.curiosity * 0.2
                + agent.skills.level(Skill::Exploration) as f32 * 0.025;
            if ambition_kind(agent) == Some(AmbitionKind::MapTheWilds) {
                willingness += 0.3;
            }
            // Every call the colony could not fill makes the next harder to
            // ignore, more so for the dutiful.
            let unanswered = view.colony.expeditions.unanswered_calls as f32;
            let duty = 1.0 + agent.personality.diligence.max(0.0);
            willingness += (unanswered * 0.06 * duty).min(0.35);
            willingness *= view.colony.volunteer_bias();
            if willingness > 0.4
                && call.volunteers.len() < game_data().balance.expeditions.max_party
            {
                out.push(Candidate {
                    goal: Goal::Volunteer,
                    utility: 0.55 + willingness * 0.3,
                    reason: reason("reason_adventure", willingness * 100.0),
                });
            }
        }
    }
}

fn aptitude(agent: &Agent, skill: Skill) -> f32 {
    let passion = if agent.skills.has_passion(skill) {
        0.3
    } else {
        0.0
    };
    let mastery = if ambition_kind(agent) == Some(AmbitionKind::MasterSkill)
        && mastery_skill(agent) == skill
    {
        0.3
    } else {
        0.0
    };
    0.6 + agent.skills.level(skill) as f32 * 0.04 + passion + mastery
}

pub fn gather_skill(kind: NodeKind) -> Skill {
    match kind {
        NodeKind::Wreckage => Skill::Crafting,
        NodeKind::FibreGrove | NodeKind::Forage => Skill::Farming,
        NodeKind::StoneOutcrop | NodeKind::OreVein => Skill::Construction,
        NodeKind::Ruin => Skill::Science,
    }
}

/// How a survivor feels about working beside whoever already pursues a goal.
fn company_factor(agent: &Agent, goal: &Goal, view: &View) -> f32 {
    let key = goal.key();
    let mut factor = 1.0;
    for other in view
        .others
        .iter()
        .filter(|s| s.id != agent.id && s.goal.is_some_and(|g| g.key() == key))
    {
        match agent.beliefs.relation(other.id).map(|r| r.bond()) {
            Some(Bond::Rival) | Some(Bond::Enemy) => factor *= 0.55,
            Some(Bond::Friend) | Some(Bond::CloseFriend) => factor *= 1.15,
            _ => factor *= 0.8,
        }
    }
    factor
}

fn work_candidates(agent: &Agent, view: &View, out: &mut Vec<Candidate>) {
    if agent.is_child {
        return;
    }
    let colony = view.colony;
    let board = &colony.jobs;
    let base = if view.is_work_hours(agent) {
        0.42 + agent.personality.diligence * 0.1
    } else {
        (0.08 + agent.personality.diligence * 0.12).max(0.02)
    };
    let fatigue = if agent.needs.get(Need::Rest) < 25.0 {
        0.5
    } else {
        1.0
    };
    let morale = if agent.mood < 30.0 { 0.8 } else { 1.0 };
    let base = base * fatigue * morale * agent.health.capacity();
    let ambition = ambition_kind(agent);
    let weight = |work: WorkType| [0.0, 0.55, 1.0, 1.5][colony.priority(work).min(3) as usize];
    let mut push = |goal: Goal, factor: f32, why: &str| {
        let utility = base * factor * company_factor(agent, &goal, view);
        if utility > 0.01 {
            out.push(Candidate {
                goal,
                utility,
                reason: game_data().label(why).to_owned(),
            });
        }
    };
    let build_bonus = if ambition == Some(AmbitionKind::BuildHome) {
        1.15
    } else {
        1.0
    };
    let actionable = |job: &&crate::colony::jobs::BuildJob| {
        job.ready_to_build
            || job.missing.nonzero().any(|(resource, _)| {
                agent.beliefs.believes_stock(resource, 1.0)
                    || view
                        .world
                        .items
                        .iter()
                        .any(|item| item.resource == resource)
            })
    };
    for job in board.builds.iter().filter(actionable).take(4) {
        let factor = weight(WorkType::Build) * aptitude(agent, Skill::Construction) * build_bonus;
        push(Goal::Build(job.site), factor, "reason_build");
    }
    for site in board.repairs.iter().take(2) {
        push(
            Goal::Repair(*site),
            weight(WorkType::Build) * aptitude(agent, Skill::Construction) * 0.8,
            "reason_repair",
        );
    }
    for farm in board.farms.iter().take(3) {
        push(
            Goal::Farm(*farm),
            weight(WorkType::Farm) * aptitude(agent, Skill::Farming),
            "reason_farm",
        );
    }
    for (kind, demand) in &board.gather_demand {
        if *demand < 0.05 || agent.beliefs.known_nodes(*kind, agent.tile).is_empty() {
            continue;
        }
        let factor = weight(WorkType::Gather) * aptitude(agent, gather_skill(*kind)) * demand;
        push(Goal::Gather(*kind), factor, "reason_gather");
    }
    let unknown_demand = board
        .gather_demand
        .iter()
        .filter(|(kind, demand)| {
            *demand > 0.3 && agent.beliefs.known_nodes(*kind, agent.tile).is_empty()
        })
        .count();
    if unknown_demand > 0 || agent.personality.curiosity > 0.3 {
        let factor = weight(WorkType::Gather)
            * 0.45
            * (1.0 + agent.personality.curiosity)
            * (1.0 + unknown_demand as f32 * 0.3);
        push(Goal::Scout, factor, "reason_scout");
    }
    if !board.items.is_empty() {
        let factor = weight(WorkType::Haul) * 0.75 * (board.items.len() as f32 / 3.0).min(1.3);
        push(Goal::Haul, factor, "reason_haul");
    }
    if board.cook_demand > 0.05 && !board.kitchens.is_empty() {
        push(
            Goal::Cook,
            weight(WorkType::Cook) * aptitude(agent, Skill::Cooking) * board.cook_demand,
            "reason_cook",
        );
    }
    for station in board.crafts.iter().take(3) {
        push(
            Goal::Craft(*station),
            weight(WorkType::Craft) * aptitude(agent, Skill::Crafting) * 0.9,
            "reason_craft",
        );
    }
    if (board.research_open || board.relic_study) && !board.labs.is_empty() {
        let decode = if ambition == Some(AmbitionKind::DecodeSpire) {
            1.3
        } else {
            1.0
        };
        let factor = weight(WorkType::Research)
            * aptitude(agent, Skill::Science)
            * (1.0 + agent.personality.curiosity * 0.5)
            * decode
            * 0.95;
        push(Goal::Research, factor, "reason_research");
    }
    for patient in board.patients.iter().filter(|p| **p != agent.id).take(2) {
        push(
            Goal::Treat(*patient),
            weight(WorkType::Medicine) * aptitude(agent, Skill::Medicine) * 1.3,
            "reason_treat",
        );
    }
    if agent.personality.bravery > 0.1 || agent.has_trait("leader") {
        for (creature, tile, _) in board
            .threats
            .iter()
            .filter(|(_, tile, _)| tile.distance(agent.tile) < 16.0)
        {
            let utility = 0.8 * (0.4 + agent.personality.bravery) * (agent.health.hp / 100.0);
            out.push(Candidate {
                goal: Goal::Defend(*creature),
                utility,
                reason: reason("reason_defend", tile.distance(agent.tile)),
            });
        }
    }
}

/// All goals worth considering now, with utilities.
pub fn candidates(agent: &Agent, view: &View) -> Vec<Candidate> {
    let mut out = Vec::new();
    if let Some((kind, until)) = agent.mental_break {
        if until > view.calendar.tick {
            let goal = match kind {
                BreakKind::LashOut => agent
                    .beliefs
                    .relations
                    .iter()
                    .min_by(|a, b| a.1.opinion.total_cmp(&b.1.opinion))
                    .map(|(id, _)| Goal::Chat(*id))
                    .unwrap_or(Goal::Break(BreakKind::Sulk)),
                other => Goal::Break(other),
            };
            out.push(Candidate {
                goal,
                utility: 10.0,
                reason: game_data().label("reason_break").to_owned(),
            });
            return out;
        }
    }
    survival_candidates(agent, view, &mut out);
    social_candidates(agent, view, &mut out);
    help_candidates(agent, view, &mut out);
    work_candidates(agent, view, &mut out);
    out.push(Candidate {
        goal: Goal::Idle,
        utility: 0.04,
        reason: String::new(),
    });
    let now = view.calendar.tick;
    out.retain(|c| !agent.is_backed_off(c.goal.key(), now));
    for candidate in out.iter_mut() {
        if candidate.goal.work_type().is_some() {
            candidate.utility /= 1.0 + view.pursuing(&candidate.goal, agent.id) as f32 * 0.35;
        }
    }
    out
}
