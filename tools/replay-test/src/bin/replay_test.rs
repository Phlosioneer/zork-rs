
extern crate replay_test;
extern crate timeout_readwrite;

#[macro_use]
extern crate log;
extern crate simplelog;

use std::fs::{self, File};
use std::io::Write;
use simplelog::{Config, LevelFilter, WriteLogger};
use replay_test::{
    Dirs, 
    parse_replay_files,
    parse_playback_files,
    fix_playback_files,
    run_playback_test, 
};

fn main() {
    WriteLogger::init(
        LevelFilter::Trace,
        Config::default(),
        File::create("replay_test_log.txt").unwrap()
    ).unwrap();

    let dirs = Dirs::new("../../").unwrap();
    debug!("dirs: {:?}", dirs);

    // Figure out the ids for the replay maps.
    let replay_map = parse_replay_files(&dirs).unwrap();

    // Find all the playback files and their ids.
    let mut playback_map = parse_playback_files(&dirs).unwrap();

    // Rename playback files as needed and filter out duplicates.
    fix_playback_files(&mut playback_map, &replay_map, &dirs);
    debug!("Playback map after fixing: {:?}", &playback_map);
    
    // Finally, make sure that we have an executable to use for testing.
    dirs.build_executable().unwrap();
    
    // Now we have our final set of playback files and their ID's. Copy them to
    // the test directory, and generate output.
    let playback_count = playback_map.len();
    for (index, (id, playback_path)) in playback_map.into_iter().enumerate() {
        
        let progress = format!("({}/{}) Running test for: {:?}", index + 1, playback_count, &playback_path);
        println!("{}", progress);
        info!("{}", progress);

        // Make the new file names.
        let dest_in_path = dirs.replay_test_dir.join(format!("in{}.txt", id));
        let dest_out_path = dirs.replay_test_dir.join(format!("out{}.txt", id));

        // Run the test to create an output file.
        let output = run_playback_test(&playback_path, &dirs).unwrap();
        
        // Save all the output.
        debug!("Saving output to {:?}", &dest_out_path);
        let mut out_file = File::create(&dest_out_path).unwrap();
        out_file.write(output.as_bytes()).unwrap();

        // Copy the old playback file into the test dir.
        // We do this after testing, so that it will not be counted as "complete"
        // if we are stopped in the middle of a test via Control-C.
        info!("Copying old playback to {:?}", &dest_in_path);
        fs::copy(&playback_path, &dest_in_path).unwrap();
       
    }
}
