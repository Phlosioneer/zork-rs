
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

lazy_static! {
    static ref PLAYBACK_FILE: Mutex<Option<File>> = Mutex::new(Some(create_playback_file()));
}

/// Creates a new file for storing the player's moves. It can be used to reproduce
/// a run through zork.
pub fn create_playback_file() -> File {
    // Get or create the "playback" directory.
    let playback_dir = Path::new("./playback");
    if !playback_dir.is_dir() {
        fs::create_dir(&playback_dir).unwrap();
    }

    // Get the next unused playback file.
    let mut filename = playback_dir.join("playback0.txt");
    let mut counter: u64 = 0;
    while filename.exists() {
        counter += 1;
        filename = playback_dir.join(format!("playback{}.txt", counter));
    }

    // Create the file and return.
    File::create(filename).unwrap()
}

/// Saves the line of input in a text file, so that it can be replayed.
pub fn record_move(player_move: &str) {
    // Note: This is a single threaded
    if let Some(ref mut file) = *PLAYBACK_FILE.try_lock().unwrap() {
        write!(file, "{}\n", &player_move).unwrap();
    }
}
