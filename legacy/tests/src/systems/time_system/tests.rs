use super::*;

#[test]
fn test_time_conversion() {
    assert_eq!(TimeSystem::get_time_of_day(0), (1, 0, 0));
    assert_eq!(TimeSystem::get_time_of_day(30), (1, 0, 30));
    assert_eq!(TimeSystem::get_time_of_day(60), (1, 1, 0));
    assert_eq!(TimeSystem::get_time_of_day(1439), (1, 23, 59));
    assert_eq!(TimeSystem::get_time_of_day(1440), (2, 0, 0));
}
