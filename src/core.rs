

use std::io::{self, Write};
use std::process::{self, Command, Stdio};
use log;
use replacement::{PromptType, supp};
use recording;

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
    supp::more_input();

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
        recording::record_move(&ret);

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
