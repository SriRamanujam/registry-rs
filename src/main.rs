mod hive;

use std::env;
use std::process;
use std::error::Error;
use std::io::prelude::*;

enum StatusCodes {
    HivePathNotPassed = 1,
    HiveError
}

fn main() {
    let mut stderr = std::io::stderr();

    if env::args().len() < 2 {
        writeln!(&mut stderr, "Need to pass path as argument").unwrap();
        process::exit(StatusCodes::HivePathNotPassed as i32);
    }

    let hive_fs_str = env::args().nth(1).unwrap();
    let h = match hive::Hive::new(hive_fs_str.as_str()) {
        Err(why) => {
            writeln!(&mut stderr, "Error opening hive: {}", why.description())
                .unwrap();
            process::exit(StatusCodes::HiveError as i32);
        },

        Ok(hive) => hive
    };
}
