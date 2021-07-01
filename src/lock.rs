use std::fs;
use std::io::{Read, Write};
use std::path::Path;

pub struct LockManager {
    locked_files: Vec<String>,
}

impl LockManager {
    pub const fn new() -> Self {
        Self {
            locked_files: Vec::new(),
        }
    }

    pub fn try_lock(&mut self, fname: &str) -> bool {
        if self.locked_files.iter().any(|n| n.as_str() == fname) {
            return true;
        }
        match do_lock(fname) {
            DoLockResult::Locked => {
                self.locked_files.push(fname.to_string());
                true
            }
            DoLockResult::Error => false,
        }
    }

    pub fn release_locks(&mut self) {
        for fname in self.locked_files.drain(..) {
            let _ = undo_lock(&fname);
        }
    }
}

enum DoLockResult {
    Locked,
    Error,
}

fn can_acquire_lock(lname: &str) -> bool {
    if let Ok(meta) = fs::metadata(lname) {
        if !meta.is_file() {
            return false;
        }
    }
    let path = Path::new(lname);
    if path.exists() {
        if let Ok(mut f) = fs::File::open(lname) {
            let mut locker = String::new();
            if f.read_to_string(&mut locker).is_ok() && !locker.is_empty() {
                return false;
            }
        }
        let _ = fs::remove_file(lname);
    }
    true
}

fn write_lock_file(lname: &str) -> DoLockResult {
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(lname)
        .map_or(DoLockResult::Error, |mut f| {
            let host = hostname();
            let content = format!("{}@{}", whoami(), host);
            let _ = f.write_all(content.as_bytes());
            DoLockResult::Locked
        })
}

fn do_lock(fname: &str) -> DoLockResult {
    let lname = format!("{fname}.lock~");
    if !can_acquire_lock(&lname) {
        return DoLockResult::Error;
    }
    write_lock_file(&lname)
}

fn undo_lock(fname: &str) -> Result<(), std::io::Error> {
    let lname = format!("{fname}.lock~");
    match fs::remove_file(&lname) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => Ok(()),
        Err(e) => Err(e),
    }
}

fn whoami() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string())
}

fn hostname() -> String {
    std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("HOST"))
        .unwrap_or_else(|_| "localhost".to_string())
}

#[cfg(test)]
mod tests;
