use super::*;

#[test]
fn test_reference_playthrough_reaches_day_7_victory_window() {
    let report = PlaytestSystem::run_reference_playthrough();

    assert_eq!(report.start_tick, REFERENCE_START_TICK);
    assert_eq!(report.strategy, PlaytestStrategyKind::Reference);
    assert_eq!(report.end_tick, TimeSystem::ticks_per_day() * 6);
    assert!(
        report.proves_reference_run(),
        "reference playthrough did not prove a full run: {:?}",
        report
    );
    assert!(report.technologies_unlocked >= report.required_technologies);
}

#[test]
fn test_strategy_variants_track_win_limp_or_fail() {
    let conservative = PlaytestSystem::run_strategy_playthrough(PlaytestStrategyKind::Conservative);
    let survey = PlaytestSystem::run_strategy_playthrough(PlaytestStrategyKind::SurveyHeavy);
    let recovery = PlaytestSystem::run_strategy_playthrough(PlaytestStrategyKind::RecoveryHeavy);

    assert_ne!(conservative.outcome_band(), PlaytestOutcomeBand::Fail);
    assert_ne!(survey.outcome_band(), PlaytestOutcomeBand::Fail);
    assert_ne!(recovery.outcome_band(), PlaytestOutcomeBand::Fail);
    assert_eq!(conservative.outcome, ScenarioOutcome::Victory);
    assert_eq!(survey.outcome, ScenarioOutcome::Victory);
    assert_eq!(recovery.outcome, ScenarioOutcome::Victory);
}

#[test]
fn test_poor_planning_variants_can_fail_day_7() {
    let no_food = PlaytestSystem::run_strategy_playthrough(PlaytestStrategyKind::NoFood);
    assert_eq!(no_food.outcome_band(), PlaytestOutcomeBand::Limp);
    assert_ne!(no_food.outcome, ScenarioOutcome::Victory);

    let no_habitats = PlaytestSystem::run_strategy_playthrough(PlaytestStrategyKind::NoHabitats);
    assert_eq!(no_habitats.outcome_band(), PlaytestOutcomeBand::Limp);
    assert_ne!(no_habitats.outcome, ScenarioOutcome::Victory);

    {
        let kind = PlaytestStrategyKind::NoMissions;
        let report = PlaytestSystem::run_strategy_playthrough(kind);
        assert_eq!(
            report.outcome_band(),
            PlaytestOutcomeBand::Fail,
            "{kind:?} should fail: {report:?}"
        );
    }
}

#[test]
fn test_playthrough_report_markdown_includes_strategy_matrix() {
    let reports = vec![
        PlaytestSystem::run_strategy_playthrough(PlaytestStrategyKind::Reference),
        PlaytestSystem::run_strategy_playthrough(PlaytestStrategyKind::NoMissions),
    ];

    let markdown = PlaytestSystem::playthrough_report_markdown(&reports);

    assert!(markdown.contains("# The Final Landing Playthrough Capture"));
    assert!(markdown.contains("| Reference | Win | Victory |"));
    assert!(markdown.contains("| No missions | Fail |"));
    assert!(markdown.contains("Tech"));
    assert!(markdown.contains("Incidents"));
}
