
use std::slice;
use libc::c_int;
use replacement::supp;
use ffi;
use ffi::objects::ObjectEntry;
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
    trace!("rdline_(*mut u8, {})", who);

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

    // Reset the state of the lexer.
    // TODO: Why 1, instead of 0?
    trace!("Modifying global variable prsvec_.prscon");
    unsafe {
        ffi::parse_vec.parse_continue = 1;
    }

    // Return via the mutated buffer.
}

// Get object.
//
// Finds an object in the room or in the inventory based on the noun(s) and
// verb(s) used to describe it. It returns the object ID if it found something.
// The id is negative if multiple objects matching that description were found.
// It returns the special value -10,000 sometimes // TODO: When...?
//
// noun is the index of a word entry in the ovoc array, NOT an object
// id!
#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn getobj_(noun: c_int, adjective: c_int, special_object: c_int) -> c_int {
    trace!("getobj_({}, {}, {})", noun, adjective, special_object);

    let CHOMP_RETURN = -10000;

    let adventurers = unsafe { &ffi::adventurers };
    let player = unsafe { ffi::player.clone() };
    let objects = unsafe { &ffi::objects };
    
    let current_adventurer = adventurers.get(player.winner as usize);

    let vehicle = current_adventurer.vehicle;

    debug!("player: {:?}", player);
    debug!("current_adventurer: {:?}", current_adventurer);
    debug!("vehicle: {:?}", vehicle);

    let mut chomp = false;
    let mut ret: Option<u32> = None;

    // Check if the current room is lit.
    let is_lit: bool = unsafe { ffi::lit_(player.current_room).into() };

    if is_lit {
        // Search the room.
        debug!("Searching room.");
        let object_id = schlst_(noun, adjective, player.current_room, 0, 0, special_object);

        if object_id < 0 {
            debug!("Found multiple objects, returning object {}.", object_id.abs());
            return object_id;
        } else if object_id > 0 {
            // Get the object.
            let current_object = objects.get(object_id as usize);

            // It's here. Is it reachable?
            if vehicle == 0 || vehicle == object_id || current_object.get_find_bit() {
                // We can reach it.
            } else if current_object.container == vehicle {
                // We can reach it.
            } else {
                // It's here and we can't reach it?
                chomp = true;
            }

            ret = Some(object_id as u32);
        }
    }

    // Not in the room. 
    // Search the vehicle.
    if vehicle != 0 {
        debug!("Searching vehicle");
        let object_id = schlst_(noun, adjective, 0, vehicle, 0, special_object);

        if object_id < 0 {
            if chomp {
                trace!("getobj_ returning {}", CHOMP_RETURN);
                return CHOMP_RETURN;
            } else {
                debug!("Found multiple objects, returning object {}.", object_id.abs());
                return object_id;
            }
        } else if object_id > 0 {
            chomp = false;
            // Did we find a different object?
            if let Some(other_id) = ret {
                if object_id != other_id as i32 {
                    debug!("Found multiple objects, returning object {}.", object_id);
                    return object_id * -1;
                }
            }

            ret = Some(object_id as u32);
        }
    }

    // Not in vehicle.
    // Search the adventurer.
    debug!("Seraching adventurer's inventory.");
    let object_id = schlst_(noun, adjective, 0, 0, player.winner, special_object);

    if object_id < 0 {
        if chomp {
            trace!("getobj_ returning {}", -10000);
            return -10000;
        } else {
            debug!("Found multiple objects, returning object {}.", object_id.abs());
            return object_id;
        }
    } else if object_id > 0 {
        
        // Did we find a different object?
        if let Some(other_id) = ret {
            if object_id != (other_id as c_int) {
                debug!("Found multiple objects, returning object {}.", object_id.abs());
                return object_id * -1;
            }
        }
        
        ret = Some(object_id as u32);
    }

    if let Some(object_id) = ret {
        debug!("Found object {}", object_id);
        return object_id as c_int;
    }

    // Search globals.
    debug!("Searching globals.");
    for global_object in objects.global_ids() {
        let object_id = global_object.get_id();

        let global_match: bool = unsafe {
            ffi::thisit_(noun, adjective, object_id as c_int, special_object).into()
        };

        trace!("Calling ghere_({}, {})", object_id, player.current_room);
        let can_be_here: bool = unsafe {
            ffi::ghere_(object_id as c_int, player.current_room).into()
        };
        trace!("ghere_ returned {}", can_be_here);

        if global_match && can_be_here {
            if ret.is_some() {
                debug!("Found multiple objects, returning object {}.", object_id);
                return (object_id as c_int) * -1;
            }

            ret = Some(object_id as u32);
        }
    }

    if let Some(object_id) = ret {
        debug!("Found object {}", object_id);
        return object_id as c_int;
    } else {
        debug!("No object found.");
        return 0;
    }
}

// Search for an object. Searches the given room, vehicle, and adventurer inventory,
// and return the corresponding object ID, or 0 if nothing matching is found.
//
// This will also check if the special_object matches, and return that if it can't
// find anything else.
//
// If there are multiple possible matches, this returns a negative object ID of
// one of the matches.
//
// This is never actually used to search multiple places at the same time, so I've
// made using multiple places an error.
#[no_mangle]
pub extern "C" fn schlst_(noun: c_int, adjective: c_int, room: c_int,
                          vehicle: c_int, adventurer: c_int, special_object: c_int) -> c_int
{
    trace!("schlst_({}, {}, {}, {}, {}, {})", noun, adjective, room, vehicle,
            adventurer, special_object);

    match (room, vehicle, adventurer) {
        (0, 0, 0) => {
            error!("schlist_ must search something.");
            core::exit_program()
        },
        (id, 0, 0) => search_room(noun, adjective,
                                  special_object as usize, id).unwrap_or(0),
        (0, id, 0) => search_container(noun, adjective,
                                       special_object as usize, id as usize).unwrap_or(0),
        (0, 0, id) => search_adventurer(noun, adjective,
                                        special_object as usize, id).unwrap_or(0),
        _ => {
            error!("Cannot schlist_ multiple places!");
            core::exit_program()
        }
    }
}

fn search_room(noun: c_int, adjective: c_int,
               special_object: usize, room: c_int) -> Option<c_int> {
    trace!("Redirecting schlst_ to room search...");
    let room_filter = |object: &ObjectEntry| object.is_in_room(room as usize);
    search_objects(noun, adjective, special_object, room_filter)
}

fn search_container(noun: c_int, adjective: c_int, 
                    special_object: usize, container: usize) -> Option<c_int> {
    trace!("Redirecting schlst_ to container search...");
    let container_filter = |object: &ObjectEntry| object.container == (container as c_int);
    search_objects(noun, adjective, special_object, container_filter)
}

fn search_adventurer(noun: c_int, adjective: c_int,
                     special_object: usize, adventurer: c_int) -> Option<c_int> {
    trace!("Redirecting schlst_ to adventurer search...");
    let adventurer_filter = |object: &ObjectEntry| object.adventurer == adventurer;
    search_objects(noun, adjective, special_object, adventurer_filter)
}

// Looks through all objects for ones that match the given filter.
fn search_objects<F>(noun: c_int, adjective: c_int, 
                     special_object: usize, f: F) -> Option<c_int>
where F: Fn(&ObjectEntry) -> bool
{
    trace!("search_objects({}, {}, {}, F)", noun, adjective, special_object);

    let objects = unsafe { &ffi::objects };

    let filtered_objects: Vec<_> = objects.iter()
        .filter(|object| object.is_visible() && f(&object))
        .collect();
   
    debug!("filtered objects: {:#?}", &filtered_objects);

    // Look for direct matches.
    let mut matches: Vec<_> = filtered_objects.iter()
        .filter(|object| {
            object.get_id() == special_object
                || object.matches(noun as usize, adjective as usize)
        })
        .map(|object| {
            let ret = object.get_id() as c_int;
            ret
        })
        .collect();

    debug!("direct matches: {:?}", &matches);

    // Look for indirect matches.
    let mut indirect_matches: Vec<_> = filtered_objects.into_iter()
        .filter(|object| object.is_open() || object.is_transparent() ||
               object.is_searchable())
        .flat_map(|object| search_container(noun, adjective, special_object, object.get_id()))
        .collect();

    debug!("indirect matches: {:?}", &indirect_matches);

    // Collect all the matches together.
    matches.append(&mut indirect_matches);

    if matches.len() == 0 {
        trace!("search_objects: No objects found.");
        None
    } else if matches.len() == 1 {
        trace!("search_objects: Returning {}", matches[0]);
        Some(matches[0])
    } else {
        trace!("search_objects: Multiple matches found: {:?}", matches);
        Some(matches[0].abs() * -1)
    }
}



// The lexer.
// c_output will always have a fixed length of 40.
// op is a pointer to a single c_int.
//
// This lexer will turn words into an array of "opcodes", stored in the
// c_output array. When it returns, op will contain the last valid index
// in the opcodes array.
//
// Lexing resumes from the character index in ParseVec::parse_continue
/*pub extern "C" fn lex_(
    c_input: *const c_char, 
    c_output: *mut c_int, 
    op: *mut c_int,
    verbose_flag: Logical
) -> Logical {
    let input = unsafe { CStr::from_ptr(c_input) };
    let mut output = unsafe { slice::from_raw_parts_mut(c_output, 40) };
    let verbose = verbose_flag.into();

    // Zero the output buffer.
    for i in output[..] {
        i = 0;
    }

    let checked_input = input.to_str().unwrap();

    trace!("Starting lexer with string: {:?}", &checked_input);

    let mut last_op: c_int = 1;
    let mut cp = 0;
    let mut ret = false;

    // TODO: Properly start at the character index given by parse_vec::parse_continue.
    for (index, c) in checked_input.char_indicies() {
        // See if we're finished with the current command.
        if c == '.' || c == ',' {
            if cp == 0 && last_op == 1 {
                // We have read no opcodes and no letters.
                // TODO: I'm pretty sure the only possible value of ret here is
                // false.
                return ret.into();
            } else if cp == 0 {
                // We have read no letters since the last opcode.
                // 
                // TODO: Why -2? Shouldn't this just trim the last one?
                //
                // Theory: It's adjusting from "index of the last op" to
                // "number of ops", in addition to trimming the last one.
                // If this is correct, then opcode 0 means "no ops were found",
                // if ret == true in the previous if statement.
                last_op -= 2;
            }
            return true.into();
        }

        // Check if this is a space.
        if c == ' ' {
            if cp == 0 {
                // Still no word found.
                continue;
            }

            // Word found; advance to next opcode spot.
            status += 2;
            op = 0;
        }

        // Check if this is a valid character.
        if c.is_alpha() || (c.is_digit() && c != '0') || c == '-' {
            if cp >= 6 {
                // Word is too long; skip until the next space is read
                // (which will reset cp to 0).
                continue;
            }
            
            // "Compute word index"?
            // This procedure seems to be storing between 1 and 3 characters per
            // "opcode". Words can be 1 to 5 characters long, so they can be either
            // 1 or 2 opcodes long.
            //
            // Note: Intentional integer division.
            let k = last_op + cp / 3;
            
            let converted_char = char_to_number(c);
            
            let opcode_adjustment = match cp {
                0 | 3 => converted_char * 1600,
                1 | 4 => converted_char * 39,
                2 => converted_char,
                _ => unreachabl!()
            }

            output[k] += opcode_adjustment;

            last_op ++;

        }

        // Invalid character.
        debug!("Invalid character: {}", c);

        // Check if we should be verbose.
        if verbose {
            trace!("Calling rspeak_(601)");
            unsafe! {
                // This prints some error message about an invalid character.
                ffi::rspeak_(601);
            }
        }
    }

    // We've reached the end of the input string. Reset the state of the lexer.
    // TODO: Why 1, instead of 0?
    unsafe {
        ffi::parse_vec.parse_continue = 1;
    }
    
    if cp == 0 && status == 1 {
        // We have read no opcodes and no letters.
        // TODO: I'm pretty sure the only way ret is true here, is if
        // the prompt was blank.
        return ret.into();
    } else if cp == 0 {
        // We have read no letters since the last opcode.
        // TODO: Why -2? Shouldn't this just trim the last one?
        // Theory: It's adjusting from "index of the last op" to
        // "number of ops", in addition to trimming the last one.
        status -= 2;
    }
    return true.into();
}
*/





