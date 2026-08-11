use super::*;
use crate::data::colonist::Trait;
use crate::data::types::Position;

fn roster_colonists(count: u32) -> Vec<Colonist> {
    (0..count)
        .map(|id| {
            Colonist::new(
                id,
                format!("Colonist {}", id),
                Position::new(id as i32, 0),
                Trait::HardWorker,
                JobPreference::Builder,
            )
        })
        .collect()
}

#[test]
fn test_assign_visible_indices_pin_selected_colonist_first() {
    let colonists = roster_colonists(6);

    assert_eq!(
        assign_visible_colonist_indices(
            &colonists,
            Some(5),
            0,
            AssignRosterFilter::All,
            AssignRosterSort::Roster,
            None,
            None,
        ),
        vec![5, 0, 1, 2, 3]
    );
    assert_eq!(
        assign_visible_colonist_indices(
            &colonists,
            None,
            0,
            AssignRosterFilter::All,
            AssignRosterSort::Roster,
            None,
            None,
        ),
        vec![0, 1, 2, 3, 4]
    );
}

#[test]
fn test_assign_visible_indices_page_through_remaining_colonists() {
    let colonists = roster_colonists(8);

    assert_eq!(
        assign_roster_page_count(&colonists, Some(5), AssignRosterFilter::All, None, None),
        2
    );
    assert_eq!(
        assign_visible_colonist_indices(
            &colonists,
            Some(5),
            1,
            AssignRosterFilter::All,
            AssignRosterSort::Roster,
            None,
            None,
        ),
        vec![5, 4, 6, 7]
    );
    assert_eq!(
        assign_visible_colonist_indices(
            &colonists,
            None,
            1,
            AssignRosterFilter::All,
            AssignRosterSort::Roster,
            None,
            None,
        ),
        vec![5, 6, 7]
    );
}

#[test]
fn test_assign_visible_indices_filter_and_sort_pressure() {
    let mut colonists = roster_colonists(6);
    colonists[1].relationships.insert(2, -12);
    colonists[3].relationships.insert(4, -34);
    colonists[4].relationships.insert(3, 22);

    assert_eq!(
        assign_visible_colonist_indices(
            &colonists,
            Some(5),
            0,
            AssignRosterFilter::Risk,
            AssignRosterSort::Bond,
            None,
            None,
        ),
        vec![5, 3, 1]
    );
}

#[test]
fn test_assign_visible_indices_filter_role() {
    let mut colonists = roster_colonists(6);
    colonists[1].job_preference = JobPreference::Cook;
    colonists[4].job_preference = JobPreference::Cook;

    assert_eq!(
        assign_visible_colonist_indices(
            &colonists,
            Some(5),
            0,
            AssignRosterFilter::All,
            AssignRosterSort::Roster,
            Some(JobPreference::Cook),
            None,
        ),
        vec![5, 1, 4]
    );
}

#[test]
fn test_assign_visible_indices_filter_building_instance() {
    let mut colonists = roster_colonists(6);
    colonists[1].assigned_habitat = Some(7);
    colonists[3].assigned_workplace = Some(7);
    colonists[4].assigned_habitat = Some(8);

    assert_eq!(
        assign_visible_colonist_indices(
            &colonists,
            Some(5),
            0,
            AssignRosterFilter::All,
            AssignRosterSort::Roster,
            None,
            Some(7),
        ),
        vec![5, 1, 3]
    );
}
