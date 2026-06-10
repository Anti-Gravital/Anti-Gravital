//! Runtime coverage for generated regex validation.

use ag_generated_rust_fixture::types::UpdateUser;

#[test]
fn generated_regex_validation_accepts_and_rejects_values() {
    let valid = UpdateUser {
        email: "user@example.com".to_owned(),
        username: "valid_user".to_owned(),
    };
    assert!(valid.validate().is_empty());

    let invalid = UpdateUser {
        email: "user@example.com".to_owned(),
        username: "INVALID USER".to_owned(),
    };
    assert_eq!(
        invalid.validate(),
        vec!["username: value does not match required pattern"]
    );
}
