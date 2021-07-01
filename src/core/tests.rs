use super::*;

#[test]
fn test_error_partial_eq() {
    assert_eq!(Error::Abort, Error::Abort);
    assert_eq!(Error::IoError, Error::IoError);
    assert_ne!(Error::Abort, Error::IoError);
}

#[test]
fn test_error_from_io() {
    let io_err = std::io::Error::other("test");
    let err: Error = io_err.into();
    assert_eq!(err, Error::IoError);
}

#[test]
fn test_result_type() {
    let ok: Result<i32> = Ok(42);
    assert!(ok.is_ok());
}

#[test]
fn test_newtype_ids() {
    assert_eq!(LineId(5).0, 5);
    assert_eq!(BufferId(3).0, 3);
    assert_eq!(WindowId(7).0, 7);
    assert_eq!(LineOffset(0).0, 0);
}

#[test]
fn test_newtype_default() {
    assert_eq!(LineId::default(), LineId(0));
    assert_eq!(BufferId::default(), BufferId(0));
}
