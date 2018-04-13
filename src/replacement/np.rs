
use std::slice;
use libc::c_int;
use replacement::supp;
use ffi;
use core;

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
        supp::exit_();
    }

    let typed_buffer = unsafe { slice::from_raw_parts_mut(buffer, 78) };

    let mut input = core::read_line(who.into());

    // Move up to 77 bytes into the buffer, after trimming whitespace.
    input.truncate(77);
    let temp_vec = vec![0];
    let iter = input.bytes().chain(temp_vec.into_iter());
    for (index, c) in iter.enumerate() {
        typed_buffer[index] = c;
    }

    // Modify some global state.
    // No idea what this does.
    trace!("Modifying global variable prsvec_.prscon");
    unsafe {
        ffi::prsvec_.prscon = 1;
    }

    // Return via the mutated buffer.
}



