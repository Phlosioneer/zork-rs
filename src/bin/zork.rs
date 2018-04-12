

extern crate zork;

#[macro_use]
extern crate log;
extern crate simplelog;

use std::fs::File;
use simplelog::{CombinedLogger, Config, LevelFilter, WriteLogger};

fn main() {
    CombinedLogger::init(vec![
        WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            File::create("log.txt").unwrap(),
        ),
        WriteLogger::new(
            LevelFilter::Trace,
            Config::default(),
            File::create("trace_log.txt").unwrap(),
        ),
    ]).unwrap();

    trace!("Starting c_main()");

    unsafe {
        zork::ffi::c_main();
    }
}


