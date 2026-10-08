use super::*;

#[test]
fn test_social_archive_markdown_exports_latest_report_first() {
    let history = vec![
        SocialHistoryEntry::new(
            1,
            "Early friction",
            "Alice and Fiona need space.",
            "Use Apart before the next work block.",
            SocialHistoryMetrics {
                average_mood: 46.0,
                average_relationship: -8.0,
                close_pairs: 0,
                strained_pairs: 1,
            },
        ),
        SocialHistoryEntry::new(
            2,
            "Shared meal",
            "Bob and Diana stabilized dinner.",
            "Keep the supportive pair together.",
            SocialHistoryMetrics {
                average_mood: 62.0,
                average_relationship: 12.0,
                close_pairs: 1,
                strained_pairs: 0,
            },
        ),
    ];

    let export = social_archive_markdown(&history);

    assert!(export.contains("# The Final Landing Social Archive"));
    assert!(export.contains("Reports: 2"));
    assert!(export.find("Day 2").unwrap() < export.find("Day 1").unwrap());
    assert!(export.contains("Recommendation: Keep the supportive pair together."));
}
