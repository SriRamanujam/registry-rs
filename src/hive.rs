use std::io;
use std::fmt;
use std::error;
use std::result;
use std::fs::File;
use std::path::Path;
use std::io::prelude::*;

#[derive(Debug)]
pub enum HiveError {
    CannotOpenHive(io::Error),
    CannotReadData(io::Error),
    InvalidHive
}

impl fmt::Display for HiveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            HiveError::CannotOpenHive(ref err) =>
                write!(f, "Unable to open hive path: {}", err),
            HiveError::CannotReadData(ref err) =>
                write!(f, "Could not read data: {}", err),
            HiveError::InvalidHive =>
                write!(f, "Invalid or corrupt hive"),
        }
    }
}

impl error::Error for HiveError {
    fn description(&self) -> &str {
        match *self {
            HiveError::CannotOpenHive(ref err) => err.description(),
            HiveError::CannotReadData(ref err) => err.description(),
            HiveError::InvalidHive => "Invalid or corrupt hive",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            HiveError::CannotOpenHive(ref err) => Some(err),
            HiveError::CannotReadData(ref err) => Some(err),
            HiveError::InvalidHive => None,
        }
    }
}

pub struct Hive {
    f: File,
}

impl Hive {
    pub fn new<P: AsRef<Path>>(path: P) -> result::Result<Hive, HiveError> {
        // open file
        let mut reg_file = try!(File::open(path.as_ref())
                      .map_err(HiveError::CannotOpenHive));

        // validate header as best we can
        let mut check = [0; 4];
        let correct_header: [u8; 4] = [0x72, 0x65, 0x67, 0x66]; //"regf" ascii chars

        try!(reg_file.read_exact(&mut check)
             .map_err(HiveError::CannotReadData));

        if !check.eq(&correct_header) {
            return Err(HiveError::InvalidHive);
        }

        Ok(Hive {f: reg_file})
    }
}
