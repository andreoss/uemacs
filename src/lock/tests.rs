use super::*;

#[test]
fn test_lock_rel() {
    let fname = "/tmp/uemacs_lock_test.txt";
    let _ = std::fs::remove_file(format!("{fname}.lock~"));

    let mut lm = LockManager::new();
    assert!(lm.try_lock(fname));
    assert!(lm.try_lock(fname));

    let lname = format!("{fname}.lock~");
    assert!(Path::new(&lname).exists());

    lm.release_locks();
    assert!(!Path::new(&lname).exists());
}

#[test]
fn test_lock_file_contains_owner() {
    let fname = "/tmp/uemacs_lock_owner.txt";
    let _ = std::fs::remove_file(format!("{fname}.lock~"));

    let mut lm = LockManager::new();
    assert!(lm.try_lock(fname));

    let lname = format!("{fname}.lock~");
    let content = std::fs::read_to_string(&lname).unwrap();
    assert!(content.contains('@'));

    lm.release_locks();
}

#[test]
fn test_undo_lock_removes_file() {
    let fname = "/tmp/uemacs_undo_lock.txt";
    let lname = format!("{fname}.lock~");
    std::fs::write(&lname, b"test").unwrap();
    assert!(Path::new(&lname).exists());

    assert!(undo_lock(fname).is_ok());
    assert!(!Path::new(&lname).exists());
}

#[test]
fn test_undo_lock_nonexistent() {
    let fname = "/tmp/uemacs_undo_nonexist.txt";
    let _ = std::fs::remove_file(format!("{fname}.lock~"));
    assert!(undo_lock(fname).is_ok());
}

#[test]
fn test_lock_twice_returns_true() {
    let fname = "/tmp/uemacs_lock_twice.txt";
    let _ = std::fs::remove_file(format!("{fname}.lock~"));

    let mut lm = LockManager::new();
    assert!(lm.try_lock(fname));
    assert!(lm.try_lock(fname));

    lm.release_locks();
}

#[test]
fn test_whoami_returns_nonempty() {
    let name = whoami();
    assert!(!name.is_empty());
}

#[test]
fn test_hostname_returns_nonempty() {
    let host = hostname();
    assert!(!host.is_empty());
}

#[test]
fn test_lock_manager_new_is_empty() {
    let lm = LockManager::new();
    assert!(lm.locked_files.is_empty());
}

#[test]
fn test_relock_reacquires() {
    let fname = "/tmp/uemacs_relock.txt";
    let _ = std::fs::remove_file(format!("{fname}.lock~"));

    let mut lm = LockManager::new();
    assert!(lm.try_lock(fname));
    lm.release_locks();

    let mut lm2 = LockManager::new();
    assert!(lm2.try_lock(fname));
    lm2.release_locks();
}
