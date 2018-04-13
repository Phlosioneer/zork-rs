
use ffi;
use core;
use std::io::{self, Write};
use std::ffi::CStr;
use libc::c_char;

// Exit the game using exit(0),
#[no_mangle]
pub extern "C" fn exit_() -> ! {
    trace!("exit_()");

    core::exit_program()
}

// Prints a character.
#[no_mangle]
pub extern "C" fn supp_putchar(c: c_char) {
    let utf: char = (c as u8).into();
    print!("{}", utf);
    io::stdout().flush().unwrap();
}

// Prints a given string.
#[no_mangle]
pub extern "C" fn more_output(out: *const c_char) {
    // If out is not null, print it and a newline.
    if !out.is_null() {
        let string = unsafe { CStr::from_ptr(out) };
        println!("{}", &string.to_str().unwrap());
        io::stdout().flush().unwrap();
    }

    // No idea what this does.
    trace!("Modifying global variable coutput");
    unsafe {
        ffi::coutput += 1;
    }
}

// No idea what this does.
#[no_mangle]
pub extern "C" fn more_input() {
    unsafe {
        ffi::coutput = 0;
    }
}


