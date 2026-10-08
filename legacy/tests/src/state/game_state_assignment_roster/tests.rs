use super::*;

#[test]
fn test_next_assign_role_filter_cycles_assignable_roles() {
    assert_eq!(next_assign_role_filter(None), Some(JobPreference::Explorer));
    assert_eq!(
        next_assign_role_filter(Some(JobPreference::Explorer)),
        Some(JobPreference::Builder)
    );
    assert_eq!(next_assign_role_filter(Some(JobPreference::Hauler)), None);
}
