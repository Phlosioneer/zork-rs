
#[macro_use]
extern crate log;

extern crate zork;
extern crate replay_test;
extern crate simplelog;
extern crate tempfile;

use std::path::PathBuf;
use std::fs::File;
use std::io::{self, Read, Write};
use std::process::{Command, Stdio};
use simplelog::{TermLogger, LevelFilter, Config};
use replay_test::Dirs;
use tempfile::NamedTempFile;

#[test]
fn test_replays() {

    // Setup
    let dirs = Dirs::new(".").unwrap();
    TermLogger::init(LevelFilter::Trace, Config::default()).unwrap();
    info!("Test log file location: {:?}", dirs.recent_log_file());

    // Get all the replay tests.
    let replay_map = replay_test::parse_replay_files(&dirs).unwrap();

    // Sort the replays into a deterministic order.
    let mut sorted_replays: Vec<_> = replay_map.into_iter().collect();
    sorted_replays.sort_unstable_by_key(|&(key, _)| key);

    // Test with every replay.
    for (_, (in_path, out_path)) in sorted_replays {
        let new_dirs = Dirs::from_dirs(&dirs).unwrap();
        test_replay(&in_path, &out_path, &new_dirs);
    }
}

fn test_replay(in_path: &PathBuf, out_path: &PathBuf, dirs: &Dirs) {
    // Run the test.
    let output_res = replay_test::run_playback_test(&in_path, &dirs);
    
    // Copy the output logs over, before running anything that might panic.
    replay_test::copy_logs(&dirs).unwrap();

    let output = output_res.unwrap();


    // Read the replay output.
    let mut expected_output = String::new();
    File::open(&out_path).unwrap().read_to_string(&mut expected_output).unwrap();

    // Compare them.
    if output != expected_output {
        // Try to make a pretty diff.
        if output.len() > 50 {
            match run_diff(&dirs, &output, &expected_output) {
                // If there was an error, fall through.
                Err(e) => println!("Error while creating diff: {:?}", e),
                Ok(diff_output) => {
                    println!("Output mismatch; diff:\n{}", diff_output);
                    panic!("Output mismatch");
                }
            }
        }   

        println!("Output mismatch; expected:\n{}\n\n\nfound:\n{}", expected_output, output);
        panic!("Output mismatch");
    }
}

fn run_diff(dirs: &Dirs, expected: &str, actual: &str) -> Result<String, io::Error> {
    let mut expected_temp = NamedTempFile::new_in(&dirs.root_dir)?;
    let mut actual_temp = NamedTempFile::new_in(&dirs.root_dir)?;

    write!(expected_temp, "{}", expected)?;
    write!(actual_temp, "{}", actual)?;

    let child_output = Command::new("git")
        .arg("diff")
        .arg("--no-index")
        .arg(actual_temp.path())
        .arg(expected_temp.path())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()?;
    

    return Ok(String::from_utf8_lossy(&child_output.stdout).into_owned());
}



