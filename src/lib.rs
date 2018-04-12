#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

extern crate chrono;
extern crate libc;
extern crate simplelog;

#[allow(unused)]
pub mod ffi;

pub mod replacement;

use replacement::PromptType;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
use std::process::{self, Command, Stdio};
use std::sync::Mutex;

lazy_static! {
    static ref PLAYBACK_FILE: Mutex<Option<File>> = Mutex::new(Some(create_playback_file()));
}

/// Creates a new file for storing the player's moves. It can be used to reproduce
/// a run through zork.
fn create_playback_file() -> File {
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

/// Prints the prompt and reads a line of input. If the input is a shell command
/// (prefixed by "!"), execute it and read again.
pub fn read_line(who: PromptType) -> String {
    // Print the prompt.
    if who == PromptType::InCharacter {
        info!("Printing prompt");
        print!(">");
        io::stdout().flush().unwrap();
    }

    // Read from stdin until a newline.
    let mut input = String::with_capacity(80);
    let res = io::stdin().read_line(&mut input);
    if let Err(err) = res {
        error!("Error reading string: {:?}", &err);
        exit_program();
    }
    debug!("Read string: {:?}", &input);

    // Update some global variables.
    trace!("calling more_input()");
    unsafe {
        ffi::more_input();
    }

    // Ensure that the input is ascii.
    if !input.is_ascii() {
        error!("Input string is not valid ascii.");
        exit_program();
    }

    // Trim whitespace from the input.
    let trimmed = input.trim();

    // If there was no input, try again.
    if trimmed.len() == 0 {
        return read_line(who);
    }

    // Check if this is a system command.
    if trimmed.starts_with("!") {
        // Execute the command.
        execute_shell_command(&trimmed[1..]);

        // Read again.
        return read_line(who);
    } else {
        // Convert the string to uppercase.
        let mut ret = trimmed.to_string();
        ret.make_ascii_uppercase();

        // Record this line.
        record_move(&ret);

        // Return.
        ret
    }
}

/// Executes a shell command, and waits for it to return.
fn execute_shell_command(command: &str) {
    // Forward this command to the shell, minus the first char.
    trace!("Calling shell with command {:?}", &command);
    let res = Command::new("bash")
        .arg("-c")
        .arg(&command)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdin(Stdio::inherit())
        .spawn();

    // Wait for the child to finish. Ignore the exit code.
    match res {
        Ok(mut child) => {
            let status = child.wait();
            match status {
                Ok(code) => info!("Subprocess exited with code: {}", &code),
                Err(err) => info!("Error while waiting for subprocess: {:?}", &err),
            }
        }
        Err(err) => info!("Error spawning subprocess: {:?}", &err),
    }
}

/// Does some cleanup and exits the program.
pub fn exit_program() -> ! {
    println!("The game is over.\n");
    io::stdout().flush().unwrap();

    info!("Exiting game.");
    log::logger().flush();

    process::exit(0)
}

/// Saves the line of input in a text file, so that it can be replayed.
fn record_move(player_move: &str) {
    // Note: This is a single threaded
    if let Some(ref mut file) = *PLAYBACK_FILE.try_lock().unwrap() {
        write!(file, "{}\n", &player_move).unwrap();
    }
}

