use super::*;

fn center(rect: Rect) -> (f32, f32) {
    (rect.x + rect.w * 0.5, rect.y + rect.h * 0.5)
}

#[test]
fn test_assign_page_hit_zones_match_roster_controls() {
    let context = Rect::new(380.0, 500.0, 520.0, 126.0);
    let (prev_x, prev_y) = center(assign_page_previous_rect(context));
    let (next_x, next_y) = center(assign_page_next_rect(context));

    assert_eq!(
        assign_page_action_at(context, prev_x, prev_y),
        Some(PageAction::Previous)
    );
    assert_eq!(
        assign_page_action_at(context, next_x, next_y),
        Some(PageAction::Next)
    );
    assert_eq!(
        assign_page_action_at(context, context.x + 18.0, context.y + 82.0),
        None
    );
}

#[test]
fn test_assign_filter_and_sort_hit_zones_match_controls() {
    let context = Rect::new(380.0, 500.0, 520.0, 126.0);
    let (risk_x, risk_y) = center(assign_filter_rect(context, 1));
    let (bond_x, bond_y) = center(assign_sort_rect(context, 2));
    let (role_x, role_y) = center(assign_role_filter_rect(context));

    assert_eq!(
        assign_filter_at(context, risk_x, risk_y),
        Some(AssignRosterFilter::Risk)
    );
    assert_eq!(
        assign_sort_at(context, bond_x, bond_y),
        Some(AssignRosterSort::Bond)
    );
    assert!(assign_role_filter_at(context, role_x, role_y));
    assert_eq!(
        assign_filter_at(context, context.x + context.w - 20.0, context.y + 21.0),
        None
    );
}

#[test]
fn test_assign_batch_hit_zones_match_copy_controls() {
    let context = Rect::new(380.0, 500.0, 520.0, 126.0);
    let (page_home_x, page_home_y) = center(assign_batch_rect(context, 0));
    let (all_work_x, all_work_y) = center(assign_batch_rect(context, 3));

    assert_eq!(
        assign_batch_action_at(context, page_home_x, page_home_y),
        Some(AssignBatchAction::PageHome)
    );
    assert_eq!(
        assign_batch_action_at(context, all_work_x, all_work_y),
        Some(AssignBatchAction::AllWork)
    );
    assert_eq!(
        assign_batch_action_at(context, context.x + 18.0, context.y + 111.0),
        None
    );
}

#[test]
fn test_assign_selection_actions_do_not_overlap_room_filter() {
    let context = Rect::new(380.0, 400.0, 520.0, 218.0);

    assert!(!assign_role_action_rect(context).overlaps(&assign_room_filter_rect(context)));
    assert!(!assign_pair_action_rect(context).overlaps(&assign_room_filter_rect(context)));
    assert!(assign_role_action_rect(context).contains(vec2(430.0, 507.0)));
}
