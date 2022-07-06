use crate::messages::{Message, StartupMessage};

#[test]
fn test_serialize() {
    let params = vec![("user".to_owned(), "postgres".to_owned())];
    let m = StartupMessage::new(params.to_owned());
    assert_eq!(m.version, (3, 0));
    assert_eq!(m.params, params);
    assert_eq!(
        m.serialize(),
        vec![
            0, 0, 0, 23, 0, 3, 0, 0, 117, 115, 101, 114, 0, 112, 111, 115, 116, 103, 114, 101, 115,
            0, 0
        ]
    );
}
