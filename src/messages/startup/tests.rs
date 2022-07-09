use crate::messages::startup::StartupMessage;
use crate::messages::{DeserializeMessage, SerializeMessage};

#[test]
fn test_serialize() {
    let params = vec![("user".to_owned(), "postgres".to_owned())];
    let m1 = StartupMessage::new(params.to_owned());
    let bytes = vec![
        0, 0, 0, 23, 0, 3, 0, 0, 117, 115, 101, 114, 0, 112, 111, 115, 116, 103, 114, 101, 115, 0,
        0,
    ];
    assert_eq!(m1.version, (3, 0));
    assert_eq!(m1.params, params);
    assert_eq!(m1.serialize(), bytes);

    let m2 = StartupMessage::deserialize_body(bytes);
    assert_eq!(m2.version, (3, 0));
    assert_eq!(m2.params, params);
}
