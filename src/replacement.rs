use libc::c_int;
use std::slice;

use ffi;
use {exit_program, read_line};

/// Exit the game using exit(0),
#[no_mangle]
pub extern "C" fn exit_() -> ! {
    trace!("exit_()");

    exit_program()
}

/// read a line of input into the buffer. the 'who' parameter is either 0
/// or 1. if it is 1, a "roleplay" prompt is printed, indicating that it
/// expects an in-game command. otherwise, no prompt is printed, indicating
/// that it expects a meta-command (such as "do you really want to quit?")
///
/// the buffer is always at least 78 characters.
#[no_mangle]
#[cfg(unix)]
pub extern "C" fn rdline_(buffer: *mut u8, who: c_int) {
    trace!("rdline_()");

    if buffer.is_null() {
        error!("null buffer given to rdline_()");
        exit_();
    }

    // i don't understand why this doesn't have to be "let mut typed_buffer;"
    let typed_buffer;
    unsafe {
        typed_buffer = slice::from_raw_parts_mut(buffer, 78);
    }

    let mut input = read_line(who.into());

    // Move up to 77 bytes into the buffer, after trimming whitespace.
    input.truncate(77);
    let temp_vec = vec![0];
    let iter = input.bytes().chain(temp_vec.into_iter());
    for (index, c) in iter.enumerate() {
        typed_buffer[index] = c;
    }

    // Modify some global state.
    trace!("Modifying global variable prsvec_.prscon");
    unsafe {
        ffi::prsvec_.prscon = 1;
    }

    // Return via the mutated buffer.
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PromptType {
    InCharacter,
    OutOfCharacter,
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
            PromptType::InCharacter => 1,
        }
    }
}