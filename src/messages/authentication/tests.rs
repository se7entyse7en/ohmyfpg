use crate::messages::{AuthenticationSASL, Message};

#[test]
fn test_serialize() {
    let m = AuthenticationSASL::new(vec!["SCRAM-SHA-256".to_owned()]);
    assert_eq!(m.mechanisms, vec!["SCRAM-SHA-256".to_owned()]);
    assert_eq!(
        m.serialize(),
        vec![
            82, 0, 0, 0, 23, 0, 0, 0, 10, 83, 67, 82, 65, 77, 45, 83, 72, 65, 45, 50, 53, 54, 0, 0
        ]
    );
}
