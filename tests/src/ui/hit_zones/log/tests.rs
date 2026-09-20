use super::*;

fn center(rect: Rect) -> (f32, f32) {
    (rect.x + rect.w * 0.5, rect.y + rect.h * 0.5)
}

#[test]
fn test_log_page_hit_zones_match_archive_controls() {
    let context = Rect::new(380.0, 500.0, 520.0, 126.0);
    let (prev_x, prev_y) = center(log_page_previous_rect(context));
    let (next_x, next_y) = center(log_page_next_rect(context));

    assert_eq!(
        log_page_action_at(context, prev_x, prev_y),
        Some(PageAction::Previous)
    );
    assert_eq!(
        log_page_action_at(context, next_x, next_y),
        Some(PageAction::Next)
    );
    assert_eq!(
        log_page_action_at(context, context.x + 18.0, context.y + 82.0),
        None
    );
}

#[test]
fn test_log_filter_hit_zones_match_filter_controls() {
    let context = Rect::new(380.0, 500.0, 520.0, 126.0);
    let (all_x, all_y) = center(log_filter_rect(context, 0));
    let (support_x, support_y) = center(log_filter_rect(context, 2));

    assert_eq!(log_filter_at(context, all_x, all_y), Some(LogFilter::All));
    assert_eq!(
        log_filter_at(context, support_x, support_y),
        Some(LogFilter::Support)
    );
    assert_eq!(
        log_filter_at(context, context.x + 18.0, context.y + 82.0),
        None
    );
}

#[test]
fn test_log_search_hit_zones_match_search_controls() {
    let context = Rect::new(380.0, 500.0, 520.0, 126.0);
    let (search_x, search_y) = center(log_search_rect(context));
    let (clear_x, clear_y) = center(log_search_clear_rect(context));
    let (export_x, export_y) = center(log_search_export_rect(context));

    assert_eq!(
        log_search_action_at(context, search_x, search_y),
        Some(LogSearchAction::Focus)
    );
    assert_eq!(
        log_search_action_at(context, clear_x, clear_y),
        Some(LogSearchAction::Clear)
    );
    assert_eq!(
        log_search_action_at(context, export_x, export_y),
        Some(LogSearchAction::Export)
    );
    assert_eq!(
        log_search_action_at(context, context.x + 18.0, context.y + 82.0),
        None
    );
}

#[test]
fn test_log_timeline_hit_zones_match_visible_rows() {
    let context = Rect::new(380.0, 500.0, 520.0, 126.0);
    let (first_x, first_y) = center(log_timeline_row_rect(context, 0));
    let (third_x, third_y) = center(log_timeline_row_rect(context, 2));

    assert_eq!(log_timeline_row_at(context, 3, first_x, first_y), Some(0));
    assert_eq!(log_timeline_row_at(context, 3, third_x, third_y), Some(2));
    assert_eq!(log_timeline_row_at(context, 2, third_x, third_y), None);
}

#[test]
fn test_log_section_tabs_and_event_rows_are_distinct() {
    let context = Rect::new(380.0, 500.0, 520.0, 276.0);
    let (social_x, social_y) = center(log_section_rect(context, 0));
    let (events_x, events_y) = center(log_section_rect(context, 1));
    let (event_x, event_y) = center(log_event_row_rect(context, 2));

    assert_eq!(
        log_section_at(context, social_x, social_y),
        Some(LogSectionAction::Social)
    );
    assert_eq!(
        log_section_at(context, events_x, events_y),
        Some(LogSectionAction::Events)
    );
    assert_eq!(log_event_row_at(context, 3, event_x, event_y), Some(2));
}
