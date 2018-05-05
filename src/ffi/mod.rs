
use libc::{c_char, c_int};
use std::ops::Range;
use core;

pub mod objects;
use self::objects::{Objects, GlobalObjects};

pub use self::prsvec_ as parse_vec;
pub use self::objcts_ as objects;
pub use self::play_ as player;
pub use self::advs_ as adventurers;
pub use self::star_ as global_items;


#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Logical(c_int);

#[link(name = "c_zork")]
extern "C" {
    pub fn c_main();

    ///////////////////////////////////////////////////////////////////////////
    // Defined in func.h

    pub fn protected() -> Logical;
    pub fn wizard() -> Logical;

    // supp.c
    pub fn more_init();
    //pub fn more_output(out: *const c_char);
    //pub fn more_input();

    pub fn bug_(a1: c_int, a2: c_int);
    pub fn cevapp_(a1: c_int);
    pub fn cpgoto_(a1: c_int);
    pub fn cpinfo_(a1: c_int, a2: c_int);
    pub fn encryp_(a1: *const c_char, a2: *mut c_char);

    // supp.c
    // Exit the game using exit(0),
    //pub fn exit_();

    pub fn fightd_();
    pub fn game_();
    pub fn gdt_();
    pub fn gttime_(a1: *mut c_int);
    pub fn invent_(a1: c_int);
    pub fn itime_(a1: *mut c_int, a2: *mut c_int, a3: *mut c_int);
    pub fn jigsup_(a1: c_int);
    pub fn newsta_(a1: c_int, a2: c_int, a3: c_int, a4: c_int, a5: c_int);
    pub fn orphan_(a1: c_int, a2: c_int, a3: c_int, a4: c_int, a5: c_int);
    pub fn princo_(a1: c_int, a2: c_int);
    pub fn princr_(a1: Logical, a2: c_int);

    // np.c
    // Read a line of input into the buffer. The 'who' parameter is either 0
    // or 1. If it is 1, a "roleplay" prompt is printed, indicating that it
    // expects an in-game command. Otherwise, no prompt is printed, indicating
    // that it expects a meta-command (such as "Do you really want to quit?")
    //
    // The buffer is always at least 78 characters.
    //pub fn rdline_(buffer: *mut c_char, who: c_int);

    // np2.c
    //pub fn schlst_(a1: c_int, a2: c_int, a3: c_int, a4: c_int, a5: c_int, a6: c_int) -> c_int;

    pub fn rspeak_(a1: c_int);
    pub fn rspsb2_(a1: c_int, a2: c_int, a3: c_int);
    pub fn rspsub_(a1: c_int, a2: c_int);
    pub fn rstrgm_();
    pub fn savegm_();
    pub fn score_(a1: Logical);
    pub fn scrupd_(a1: c_int);
    pub fn swordd_();
    pub fn theifd_();
    pub fn valuac_(a1: c_int);

    pub fn blow_(a1: c_int, a2: c_int, a3: c_int, a4: Logical, a5: c_int) -> c_int;
    pub fn fights_(a1: c_int, a2: Logical) -> c_int;
    pub fn fwim_(a1: c_int, a2: c_int, a3: c_int, a4: c_int, a5: c_int, a6: Logical) -> c_int;
    pub fn getobj_(a1: c_int, a2: c_int, a3: c_int) -> c_int;
    pub fn mrhere_(a1: c_int) -> c_int;
    pub fn oactor_(a1: c_int) -> c_int;
    pub fn rnd_(a1: c_int) -> c_int;
    pub fn robadv_(a1: c_int, a2: c_int, a3: c_int, a4: c_int) -> c_int;
    pub fn robrm_(a1: c_int, a2: c_int, a3: c_int, a4: c_int, a5: c_int) -> c_int;
    pub fn sparse_(a1: *const c_int, a2: c_int, a3: Logical) -> c_int;
    pub fn vilstr_(a1: c_int) -> c_int;
    pub fn weight_(a1: c_int, a2: c_int, a3: c_int) -> c_int;

    pub fn aappli_(a1: c_int) -> Logical;
    pub fn ballop_(a1: c_int) -> Logical;
    pub fn clockd_() -> Logical;
    pub fn cyclop_(a1: c_int) -> Logical;
    pub fn drop_(a1: Logical) -> Logical;
    pub fn findxt_(a1: c_int, a2: c_int) -> Logical;
    pub fn ghere_(a1: c_int, a2: c_int) -> Logical;
    pub fn init_() -> Logical;
    pub fn lightp_(a1: c_int) -> Logical;
    pub fn lit_(a1: c_int) -> Logical;
    pub fn moveto_(a1: c_int, a2: c_int) -> Logical;
    pub fn nobjs_(a1: c_int, a2: c_int) -> Logical;
    pub fn oappli_(a1: c_int, a2: c_int) -> Logical;
    pub fn objact_() -> Logical;
    pub fn opncls_(a1: c_int, a2: c_int, a3: c_int) -> Logical;
    
    // np.c
    pub fn parse_(a1: *mut c_char, a2: Logical) -> Logical;
    
    pub fn prob_(a1: c_int, a2: c_int) -> Logical;
    pub fn put_(a1: Logical) -> Logical;
    pub fn rappli_(a1: c_int) -> Logical;
    pub fn rappl1_(a1: c_int) -> Logical;
    pub fn rappl2_(a1: c_int) -> Logical;
    pub fn rmdesc_(a1: c_int) -> Logical;
    pub fn sobjs_(a1: c_int, a2: c_int) -> Logical;
    pub fn sverbs_(a1: c_int) -> Logical;
    pub fn synmch_() -> Logical;
    pub fn take_(a1: Logical) -> Logical;
    pub fn thiefp_(a1: c_int) -> Logical;
    pub fn trollp_(a1: c_int) -> Logical;
    pub fn qempty_(a1: c_int) -> Logical;
    pub fn qhere_(a1: c_int, a2: c_int) -> Logical;
    pub fn vappli_(a1: c_int) -> Logical;
    pub fn walk_() -> Logical;
    pub fn winnin_(a1: c_int, a2: c_int) -> Logical;
    pub fn yesno_(a1: c_int, a2: c_int, a3: c_int) -> Logical;

    ///////////////////////////////////////////////////////////////////////////
    // Defined in vars.h

    pub static mut prsvec_: ParseVec;
    pub static mut objcts_: Objects;
    pub static mut play_: Player;
    pub static mut advs_: Adventurers;
    pub static mut star_: GlobalObjects;

    ///////////////////////////////////////////////////////////////////////////
    // Defined elsewhere

    ////////////
    // np.c
    //pub fn lex_(a1: *mut c_char, a2: *mut c_int, a3: *mut c_int, a4: Logical) -> Logical;

    ////////////
    // np2.c
    pub fn thisit_ (a1: c_int, a2: c_int, a3: c_int, a4: c_int) -> Logical;

    ////////////
    // supp.c
    pub static mut coutput: c_int;

}

// Info about the player.
#[repr(C)]
#[derive(Clone, Debug)]
pub struct Player {
    pub winner: c_int,
    pub current_room: c_int,
    pub tel_flag: Logical
}

// Info about all adventurers. (There are multiple...?)
#[repr(C)]
pub struct Adventurers {
    limit: c_int,
    
    // The current room for this adventurer.
    rooms: [c_int; 4],
    
    // The adventurer's score.
    scores: [c_int; 4],
    vehicles: [c_int; 4],
    
    // The current object referred to by the word "it".
    current_its: [c_int; 4],
    actions: [c_int; 4],
    strengths: [c_int; 4],
    flags: [c_int; 4]
}

// Info about a particular adventurer.
#[derive(Debug, Clone)]
pub struct AdventurerEntry {
    index: usize,
    
    // The current room the adventurer is in.
    pub room: c_int,
    
    // Their score.
    pub score: c_int,
    pub vehicle: c_int,

    // The current object referred to by the word "it".
    pub current_it: c_int,
    pub action: c_int,
    pub strength: c_int,
    pub flags: c_int
}

impl Adventurers {
    // Get an adventurer by their ID. Ids start at 1.
    pub fn get(&self, id: usize) -> AdventurerEntry {
        if id > self.len() {
            error!("Adventurer id {} is greater than object count ({}).", id, self.len());
            core::exit_program();
        }
        if id == 0 {
            error!("Adventurer id cannot be 0.");
            core::exit_program();
        }

        let index = id - 1;

        AdventurerEntry {
            index,
            room: self.rooms[index],
            score: self.scores[index],
            vehicle: self.vehicles[index],
            current_it: self.current_its[index],
            action: self.actions[index],
            strength: self.strengths[index],
            flags: self.flags[index]
        }
    }

    pub fn set(&mut self, values: &AdventurerEntry) {
        let index = values.index;

        self.rooms[index] = values.room;
        self.scores[index] = values.score;
        self.vehicles[index] = values.vehicle;
        self.current_its[index] = values.current_it;
        self.actions[index] = values.action;
        self.strengths[index] = values.strength;
        self.flags[index] = values.flags;
    }

    pub fn len(&self) -> usize {
        self.limit as usize
    }
}

impl AdventurerEntry {
    pub fn get_id(&self) -> usize {
        self.index + 1
    }
}


// A structure that stores info during parsing.
#[repr(C)]
#[derive(Clone, Debug)]
pub struct ParseVec {
    pub parse_a: c_int,
    pub parse_i: c_int,
    pub parse_o: c_int,
    pub parse_won: Logical,

    // Parsing should continue from this index in the parsing string. This is used
    // when an input string contains multiple commands (separated by '.' or ',').
    pub parse_continue: c_int,
}

impl From<bool> for Logical {
    fn from(other: bool) -> Logical {
        if other {
            Logical(1)
        } else {
            Logical(0)
        }
    }
}

impl From<Logical> for bool {
    fn from(other: Logical) -> bool {
        match other.0 {
            0 => false,
            1 => true,
            _ => panic!("Illegal value for Logical: {}", other.0)
        }
    }
}

/// Returns true if the input is nonzero.
pub fn c_int_to_bool(input: c_int) -> bool {
    input != 0
}

