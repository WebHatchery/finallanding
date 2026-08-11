use super::*;
use crate::data::colonist::Trait;
use crate::data::types::Position;

fn test_colonist(id: u32) -> Colonist {
    Colonist::new(
        id,
        format!("Colonist {}", id),
        Position::new(id as i32, 0),
        Trait::HardWorker,
        JobPreference::Builder,
    )
}

#[test]
fn test_assign_visible_colonists_pin_selected_first() {
    let colonists = (0..6).map(test_colonist).collect::<Vec<_>>();
    let visible = assign_visible_colonists(
        &colonists,
        Some(5),
        0,
        AssignRosterFilter::All,
        AssignRosterSort::Roster,
        None,
        None,
    )
    .into_iter()
    .map(|colonist| colonist.id)
    .collect::<Vec<_>>();

    assert_eq!(visible, vec![5, 0, 1, 2, 3]);
}

#[test]
fn test_assign_visible_colonists_page_through_roster() {
    let colonists = (0..8).map(test_colonist).collect::<Vec<_>>();
    let page = assign_visible_colonists(
        &colonists,
        Some(5),
        1,
        AssignRosterFilter::All,
        AssignRosterSort::Roster,
        None,
        None,
    )
    .into_iter()
    .map(|colonist| colonist.id)
    .collect::<Vec<_>>();

    assert_eq!(
        assign_roster_page_count(&colonists, Some(5), AssignRosterFilter::All, None, None),
        2
    );
    assert_eq!(page, vec![5, 4, 6, 7]);
}

#[test]
fn test_assign_visible_colonists_filter_pinned_and_sort_mood() {
    let mut colonists = (0..6).map(test_colonist).collect::<Vec<_>>();
    colonists[1].assigned_habitat = Some(3);
    colonists[1].mood = 42.0;
    colonists[4].assigned_workplace = Some(8);
    colonists[4].mood = 21.0;

    let visible = assign_visible_colonists(
        &colonists,
        Some(5),
        0,
        AssignRosterFilter::Pinned,
        AssignRosterSort::Mood,
        None,
        None,
    )
    .into_iter()
    .map(|colonist| colonist.id)
    .collect::<Vec<_>>();

    assert_eq!(visible, vec![5, 4, 1]);
}

#[test]
fn test_assign_visible_colonists_filter_role() {
    let mut colonists = (0..6).map(test_colonist).collect::<Vec<_>>();
    colonists[1].job_preference = JobPreference::Cook;
    colonists[3].job_preference = JobPreference::Cook;
    colonists[4].job_preference = JobPreference::Explorer;

    let visible = assign_visible_colonists(
        &colonists,
        Some(5),
        0,
        AssignRosterFilter::All,
        AssignRosterSort::Roster,
        Some(JobPreference::Cook),
        None,
    )
    .into_iter()
    .map(|colonist| colonist.id)
    .collect::<Vec<_>>();

    assert_eq!(visible, vec![5, 1, 3]);
}

#[test]
fn test_assign_visible_colonists_filter_building_instance() {
    let mut colonists = (0..6).map(test_colonist).collect::<Vec<_>>();
    colonists[1].assigned_habitat = Some(7);
    colonists[3].assigned_workplace = Some(7);
    colonists[4].assigned_habitat = Some(8);

    let visible = assign_visible_colonists(
        &colonists,
        Some(5),
        0,
        AssignRosterFilter::All,
        AssignRosterSort::Roster,
        None,
        Some(7),
    )
    .into_iter()
    .map(|colonist| colonist.id)
    .collect::<Vec<_>>();

    assert_eq!(visible, vec![5, 1, 3]);
}

#[test]
fn test_assign_pair_action_reports_active_directive() {
    let mut colonists = vec![test_colonist(1), test_colonist(2)];
    colonists[0].relationships.insert(2, -24);
    colonists[1].relationships.insert(1, -20);
    colonists[0].avoided_partner_id = Some(2);
    colonists[1].avoided_partner_id = Some(1);

    let action = assign_pair_action(&colonists, 1, 2).unwrap();

    assert_eq!(action.directive, PairDirective::Separate);
    assert_eq!(action.label, "Apart set -22");
}

#[test]
fn test_selected_assignment_label_reports_room_pins() {
    let mut colonist = test_colonist(1);
    assert_eq!(selected_assignment_label(&colonist), "H-- W--");

    colonist.assigned_habitat = Some(3);
    colonist.assigned_workplace = Some(8);

    assert_eq!(selected_assignment_label(&colonist), "H#3 W#8");
    assert!(selected_assignment_detail(
        &colonist,
        &[colonist.clone()],
        &TechnologyState::default()
    )
    .contains("H#3 W#8"));
}

#[test]
fn test_assignment_pin_warning_flags_over_capacity_habitat() {
    let mut colonists = vec![test_colonist(1), test_colonist(2), test_colonist(3)];
    for colonist in &mut colonists {
        colonist.assigned_habitat = Some(7);
    }

    let warning =
        assignment_pin_warning(&colonists[0], &colonists, &TechnologyState::default()).unwrap();

    assert_eq!(warning.label, "CAP");
    assert!(warning.detail.contains("3/2"));
}

#[test]
fn test_assignment_pin_warning_flags_tense_shared_workplace() {
    let mut colonists = vec![test_colonist(1), test_colonist(2)];
    colonists[0].assigned_workplace = Some(9);
    colonists[1].assigned_workplace = Some(9);
    colonists[0].relationships.insert(2, -24);
    colonists[1].relationships.insert(1, -20);

    let warning =
        assignment_pin_warning(&colonists[0], &colonists, &TechnologyState::default()).unwrap();

    assert_eq!(warning.label, "TENSE");
    assert!(warning.detail.contains("Colonist 2"));
    assert!(warning.detail.contains("W#9"));
}
