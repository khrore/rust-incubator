#[cfg(test)]
use crate::email::*;

#[test]
fn email_test() {
    assert!(EmailString::try_from("".to_string()).is_err());
    assert!(EmailString::try_from("test".to_string()).is_err());
    assert!(EmailString::try_from("test@".to_string()).is_err());
    assert!(EmailString::try_from("test@test".to_string()).is_err());
    assert!(EmailString::try_from("@test@test".to_string()).is_err());
    assert!(EmailString::try_from("@test@".to_string()).is_err());
    assert!(EmailString::try_from("test@test.".to_string()).is_err());
    assert!(EmailString::try_from("test@test.a".to_string()).is_err());
    assert!(EmailString::try_from("test@test.ab".to_string()).is_ok());
}

use crate::random::*;

#[test]
fn random_test() {
    let rand_val = Random::new(1, 2, 3);
    assert!([1, 2, 3].contains(&*rand_val));
}
