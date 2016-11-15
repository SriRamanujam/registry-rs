use std::io;
use std::fmt;
use std::error;
use std::result;
use std::fs::File;
use std::path::Path;
use std::io::Cursor;
use std::io::SeekFrom;
use std::io::prelude::*;

use byteorder::{LittleEndian, ReadBytesExt};

/// The HiveError type. Every sort of error is represented in this enum.
#[derive(Debug)]
pub enum HiveError {
    /// The file cannot be opened. Carries with it the underlying io::Error.
    CannotOpenHive(io::Error),
    /// Data could not be read. Carries with it the underlying io::Error.
    CannotReadData(io::Error),
    /// The hive is invalid.
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
    /// Validates the registry hive as best it can, and constructs the Hive
    /// object if all checks pass.
    ///
    /// This function checks the following in order to ensure a valid hive:
    /// 1. Whether the file signature is correct
    /// 2. Whether the primary and secondary sequence numbers are synchronized
    /// 3. Whether the header checksum is valid
    ///
    /// # Examples
    ///
    /// ```
    /// use hive::Hive;
    ///
    /// let h = Hive::open("/path/to/hive/file");
    /// ```
    ///
    /// # Errors
    /// Will return with the appropriate error if the file cannot be opened, if
    /// data could not be read, or if the file is not a valid registry hive.
    ///
    pub fn new<P: AsRef<Path>>(path: P) -> result::Result<Hive, HiveError> {
        // open file
        let mut reg_file = try!(File::open(path.as_ref())
                      .map_err(HiveError::CannotOpenHive));

        // check file signature
        let mut file_sig = [0; 4];
        let actual_sig: [u8; 4] = [0x72, 0x65, 0x67, 0x66]; //"regf" ascii chars

        try!(reg_file.read_exact(&mut file_sig)
             .map_err(HiveError::CannotReadData));

        if !file_sig.eq(&actual_sig) {
            return Err(HiveError::InvalidHive);
        }

        // check sequence numbers
        let mut primary = [0; 4];
        let mut secondary = [0; 4];
        try!(reg_file.read_exact(&mut primary)
             .map_err(HiveError::CannotReadData));
        try!(reg_file.read_exact(&mut secondary).
             map_err(HiveError::CannotReadData));

        if !primary.eq(&secondary) {
            return Err(HiveError::InvalidHive);
        }

        // do the XOR checksum
        let mut header_raw = [0; 508];
        let mut header_check = [0; 4];
        let mut xor: u32 = 0;

        try!(reg_file.seek(SeekFrom::Start(0))
             .map_err(HiveError::CannotReadData));
        try!(reg_file.read_exact(&mut header_raw)
             .map_err(HiveError::CannotReadData));
        try!(reg_file.read_exact(&mut header_check)
             .map_err(HiveError::CannotReadData));

        let mut header_rdr = Cursor::new(header_raw.to_vec());
        let mut check_rdr = Cursor::new(header_check);

        let csum = check_rdr.read_u32::<LittleEndian>().unwrap();

        for _ in 0..127 { // 508 / 4, the number of u32s in the vec
            xor ^= try!(header_rdr.read_u32::<LittleEndian>()
                        .map_err(HiveError::CannotReadData));
        }

        if xor != csum {
            return Err(HiveError::InvalidHive);
        }

        // reset the seek ptr to 0 before instantiating
        try!(reg_file.seek(SeekFrom::Start(0))
             .map_err(HiveError::CannotReadData));

        Ok(Hive {f: reg_file})
    }
}
