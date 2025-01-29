// Copyright (C) 2024 SUSE LLC <petr.pavlu@suse.com>
// SPDX-License-Identifier: GPL-2.0-or-later

use std::path::Path;

pub mod diff;
pub mod sym;

#[derive(Debug)]
pub enum Error {
    IO {
        desc: String,
        io_err: std::io::Error,
    },
    Parse(String),
}

impl Error {
    fn new_io(desc: &str, io_err: std::io::Error) -> Self {
        Error::IO {
            desc: desc.to_string(),
            io_err,
        }
    }

    fn new_parse(desc: &str) -> Self {
        Error::Parse(desc.to_string())
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::IO { desc, io_err } => {
                write!(f, "{}: ", desc)?;
                io_err.fmt(f)
            }
            Self::Parse(desc) => write!(f, "{}", desc),
        }
    }
}

/// A helper extension trait to map std::io::Error to crate::Error(), as write!(...).map_io_error().
trait MapIOErr {
    fn map_io_err(self, path: &Path) -> Result<(), crate::Error>;
}

impl MapIOErr for Result<(), std::io::Error> {
    fn map_io_err(self, path: &Path) -> Result<(), crate::Error> {
        self.map_err(|err| {
            crate::Error::new_io(
                &format!("Failed to write data to file '{}'", path.display()),
                err,
            )
        })
    }
}

pub static DEBUG_LEVEL: std::sync::OnceLock<usize> = std::sync::OnceLock::new();

pub fn init_debug_level(level: usize) {
    assert!(DEBUG_LEVEL.get().is_none());
    DEBUG_LEVEL.get_or_init(|| level);
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        if *$crate::DEBUG_LEVEL.get().unwrap_or(&0) > 0 {
            eprintln!($($arg)*);
        }
    }
}

#[cfg(test)]
#[macro_export]
macro_rules! assert_ok {
    ($result:expr) => {
        match $result {
            Ok(()) => {}
            result => panic!("assertion failed: {:?} is not of type Ok(())", result),
        }
    };
}

#[cfg(test)]
#[macro_export]
macro_rules! string_vec {
      ($($x:expr),*) => (vec![$($x.to_string()),*]);
}
