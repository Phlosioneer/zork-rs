
#[macro_use]
extern crate log;
extern crate libc;
extern crate simplelog;

mod ffi;

use std::io::{self, Write};
use std::slice;
use std::fs::File;
use libc::c_int;
use std::process::{self, Stdio, Command};
use simplelog::{WriteLogger, Config, LevelFilter, CombinedLogger};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum PromptType {
    InCharacter,
    OutOfCharacter
}

impl From<c_int> for PromptType {
    fn from(other: c_int) -> PromptType {
        if other == 0 {
            PromptType::OutOfCharacter
        } else if other == 1 {
            PromptType::InCharacter
        } else {
            panic!("Invalid conversion from c_int to PromptType: {:?}", other);
        }
    }
}

impl From<PromptType> for c_int {
    fn from(other: PromptType) -> c_int {
        match other {
            PromptType::OutOfCharacter => 0,
            PromptType::InCharacter => 1
        }
    }
}

/// Read a line of input into the buffer. The 'who' parameter is either 0
/// or 1. If it is 1, a "roleplay" prompt is printed, indicating that it
/// expects an in-game command. Otherwise, no prompt is printed, indicating
/// that it expects a meta-command (such as "Do you really want to quit?")
///
/// The buffer is always at least 78 characters.
#[no_mangle]
#[cfg(unix)]
pub extern "C" fn rdline_(buffer: *mut u8, who: c_int) {
    trace!("rdline_()");

    if buffer.is_null() {
        error!("Null buffer given to rdline_()");
        exit_();
    }

    // I don't understand why this doesn't have to be "let mut typed_buffer;"
    let typed_buffer;
    unsafe { typed_buffer = slice::from_raw_parts_mut(buffer, 78); }

    let mut input = String::new();

    loop {

        // Print the prompt.
        let converted_who: PromptType = who.into();
        info!("converted_who: {:?}", &converted_who);
        if converted_who == PromptType::InCharacter {
            info!("Printing prompt");
            print!(">");
            io::stdout().flush().unwrap();
        }

        // Read from stdin until a newline.
        input.clear();
        let res = io::stdin().read_line(&mut input);
        if let Err(err) = res {
            error!("Error reading string: {:?}", &err);
            exit_();
        } else {
            debug!("Read string: {:?}", &input);
        }

        // Update some global variables.
        trace!("calling more_input()");
        unsafe { ffi::more_input(); }

        // If there was no input, try again.
        if input.len() == 0 {
            continue;
        }

        // Check if this is a system command.
        if &input[0..1] == "!" {
            // Forward this command to the shell, minus the first char.
            trace!("Calling shell with command {:?}", &input[1..]);
            let res = Command::new("bash")
                .arg("-c")
                .arg(&input[1..])
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
                        Err(err) => info!("Error while waiting for subprocess: {:?}", &err)
                    }
                },
                Err(err) => info!("Error spawning subprocess: {:?}", &err)
            }

            // Read more.
            continue;
        }

        break;
    }

    // Convert the string to uppercase.
    input.make_ascii_uppercase();

    // Move up to 77 bytes into the buffer, after trimming whitespace.
    input.truncate(77);
    let trimmed = input.trim_right();
    let temp_vec = vec![0];
    let iter = trimmed.bytes().chain(temp_vec.into_iter());
    for (index, c) in iter.enumerate() {
        typed_buffer[index] = c;
    }
    
    // Modify some global state.
    trace!("Modifying global variable prsvec_.prscon");
    unsafe { ffi::prsvec_.prscon = 1; }

    // Return via the mutated buffer.
}


/// Exit the game using exit(0),
#[no_mangle]
pub extern "C" fn exit_() -> ! {
    trace!("exit_()");

    println!("The game is over.\n");
    io::stdout().flush().unwrap();
    
    log::logger().flush();

    process::exit(0)
}

fn main() {
    CombinedLogger::init(
        vec![
            WriteLogger::new(LevelFilter::Debug, Config::default(),
                File::create("log.txt").unwrap()),
            WriteLogger::new(LevelFilter::Trace, Config::default(),
                File::create("trace_log.txt").unwrap()),
        ]
    ).unwrap();

    trace!("Starting c_main()");

    unsafe {
        ffi::c_main();
    }
}
