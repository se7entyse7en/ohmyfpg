use crate::messages::authentication::AuthenticationSASL;
use crate::messages::{DeserializeMessage, SerializeMessage};

#[test]
fn test_serialize_deserialize() {
    let m1 = AuthenticationSASL::new(vec!["SCRAM-SHA-256".to_owned()]);
    let bytes = vec![
        82, 0, 0, 0, 23, 0, 0, 0, 10, 83, 67, 82, 65, 77, 45, 83, 72, 65, 45, 50, 53, 54, 0, 0,
    ];
    assert_eq!(m1.mechanisms, vec!["SCRAM-SHA-256".to_owned()]);
    assert_eq!(m1.serialize(), bytes);

    let m2 = AuthenticationSASL::deserialize_body(bytes[5..].to_vec());
    assert_eq!(m2.mechanisms, vec!["SCRAM-SHA-256".to_owned()]);
}

// TODO: Add tests for other Auth SASL messages
