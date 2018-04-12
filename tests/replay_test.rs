
extern crate zork;
extern crate replay_test;
extern crate simplelog;

use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use simplelog::{TermLogger, LevelFilter, Config};
use replay_test::Dirs;

#[test]
fn test_replays() {

    // Setup
    let dirs = Dirs::new(".").unwrap();
    TermLogger::init(LevelFilter::Trace, Config::default());
    
    // Get all the replay tests.
    let replays = replay_test::parse_replay_files(&dirs).unwrap();

    // Test with every replay.
    for (in_path, out_path) in replays.values() {
        let new_dirs = Dirs::from_dirs(&dirs).unwrap();
        test_replay(&in_path, &out_path, &new_dirs);
    }
}

fn test_replay(in_path: &PathBuf, out_path: &PathBuf, dirs: &Dirs) {
    // Run the test.
    let output = replay_test::run_playback_test(&in_path, &dirs).unwrap();
    
    // Read the replay output.
    let mut expected_output = String::new();
    File::open(&out_path).unwrap().read_to_string(&mut expected_output);

    // Compare them.
    if output != expected_output {
        panic!("Output mismatch; expected: {:?}\n\n\nfound: {:?}", expected_output, output);
    }
}



