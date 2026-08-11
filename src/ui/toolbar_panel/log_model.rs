use super::SOCIAL_TIMELINE_PAGE_SIZE;
use crate::data::colonist::RelationshipBand;
use crate::data::event_log::SocialHistoryEntry;
use crate::systems::summary_system::{ColonyPressureSummary, RelationshipPairSummary};
use crate::ui::hit_zones::LogFilter;
use crate::ui::style;
use macroquad::prelude::Color;

pub(super) struct SocialTimelineRow {
    pub(super) day: u32,
    pub(super) title: String,
    pub(super) detail: String,
    pub(super) metrics: String,
    pub(super) color: Color,
}

pub fn social_history_page_count(
    history: &[SocialHistoryEntry],
    filter: LogFilter,
    query: &str,
) -> usize {
    let count = history
        .iter()
        .filter(|entry| social_history_matches_filter(entry, filter))
        .filter(|entry| social_history_matches_query(entry, query))
        .count();
    count.div_ceil(SOCIAL_TIMELINE_PAGE_SIZE).max(1)
}

pub(super) fn social_timeline_rows(
    history: &[SocialHistoryEntry],
    filter: LogFilter,
    query: &str,
    page: usize,
) -> Vec<SocialTimelineRow> {
    let page = page.min(social_history_page_count(history, filter, query).saturating_sub(1));
    history
        .iter()
        .rev()
        .filter(|entry| social_history_matches_filter(entry, filter))
        .filter(|entry| social_history_matches_query(entry, query))
        .skip(page * SOCIAL_TIMELINE_PAGE_SIZE)
        .take(SOCIAL_TIMELINE_PAGE_SIZE)
        .map(|entry| SocialTimelineRow {
            day: entry.day,
            title: entry.title.clone(),
            detail: format!("{} {}", entry.detail, entry.recommendation),
            metrics: format!(
                "M{:.0} R{:+.0} T{}",
                entry.average_mood, entry.average_relationship, entry.strained_pairs
            ),
            color: social_history_color(entry),
        })
        .collect()
}

pub fn social_timeline_day_at(
    history: &[SocialHistoryEntry],
    filter: LogFilter,
    query: &str,
    page: usize,
    row_index: usize,
) -> Option<u32> {
    social_timeline_rows(history, filter, query, page)
        .get(row_index)
        .map(|row| row.day)
}

pub(super) fn selected_social_history_entry(
    history: &[SocialHistoryEntry],
    selected_day: Option<u32>,
) -> Option<&SocialHistoryEntry> {
    let day = selected_day?;
    history.iter().find(|entry| entry.day == day)
}

pub(super) fn social_history_matches_filter(entry: &SocialHistoryEntry, filter: LogFilter) -> bool {
    match filter {
        LogFilter::All => true,
        LogFilter::Tense => social_history_signal(entry) == SocialHistorySignal::Tense,
        LogFilter::Support => social_history_signal(entry) == SocialHistorySignal::Support,
    }
}

pub(super) fn social_history_matches_query(entry: &SocialHistoryEntry, query: &str) -> bool {
    let query = query.trim();
    if query.is_empty() {
        return true;
    }

    let needle = query.to_ascii_lowercase();
    entry.title.to_ascii_lowercase().contains(&needle)
        || entry.detail.to_ascii_lowercase().contains(&needle)
        || entry.recommendation.to_ascii_lowercase().contains(&needle)
        || format!("day {}", entry.day).contains(&needle)
        || entry.day.to_string().contains(&needle)
}

pub(super) fn social_history_color(entry: &SocialHistoryEntry) -> Color {
    match social_history_signal(entry) {
        SocialHistorySignal::Tense => style::ALERT_RED,
        SocialHistorySignal::Support => style::BAR_GREEN,
        SocialHistorySignal::Neutral => style::HEADING_BLUE,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SocialHistorySignal {
    Tense,
    Support,
    Neutral,
}

fn social_history_signal(entry: &SocialHistoryEntry) -> SocialHistorySignal {
    if entry.strained_pairs > 0 || entry.average_relationship < -5.0 {
        SocialHistorySignal::Tense
    } else if entry.close_pairs > 0 || entry.average_relationship > 8.0 {
        SocialHistorySignal::Support
    } else {
        SocialHistorySignal::Neutral
    }
}

pub(super) struct SocialBriefLines {
    pub(super) header: String,
    pub(super) detail: String,
    pub(super) color: Color,
}

pub(super) fn social_brief_lines(summary: &ColonyPressureSummary) -> SocialBriefLines {
    let color = if summary.strained_pairs > 0 {
        style::ALERT_RED
    } else if summary.close_pairs > 0 {
        style::BAR_GREEN
    } else {
        style::HEADING_BLUE
    };

    let header = format!(
        "Social pressure: mood {:.0} | close {} | tense {}",
        summary.average_mood, summary.close_pairs, summary.strained_pairs
    );
    let detail = if let Some(pair) = summary
        .weakest_pair
        .as_ref()
        .filter(|pair| RelationshipBand::from_value(pair.value).is_risk())
    {
        pair_line("Watch", pair)
    } else if let Some(pair) = summary
        .strongest_pair
        .as_ref()
        .filter(|pair| RelationshipBand::from_value(pair.value).is_support())
    {
        pair_line("Protect", pair)
    } else {
        "No strong social signal yet; routine will shape the first bonds.".to_string()
    };

    SocialBriefLines {
        header,
        detail,
        color,
    }
}

pub(super) fn pair_line(prefix: &str, pair: &RelationshipPairSummary) -> String {
    format!(
        "{} {} / {}: {} {:+}",
        prefix, pair.first_name, pair.second_name, pair.label, pair.value
    )
}

#[cfg(test)]
mod tests;
