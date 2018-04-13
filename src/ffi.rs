
use libc::{c_char, c_int};

#[repr(C)]
pub struct Logical(c_int);

#[link(name = "c_zork")]
extern "C" {
    pub fn c_main();

    ///////////////////////////////////////////////////////////////////////////
    // Defined in func.h

    pub fn protected() -> Logical;
    pub fn wizard() -> Logical;

    //pub fn more_init();
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
    pub fn schlst_(a1: c_int, a2: c_int, a3: c_int, a4: c_int, a5: c_int, a6: c_int) -> c_int;
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

    pub static mut prsvec_: PrsVec;

    ///////////////////////////////////////////////////////////////////////////
    // Defined elsewhere

    ////////////
    // np.h
    pub static mut lex_:
        extern "C" fn(a1: *mut c_char, a2: *mut c_int, a3: *mut c_int, a4: Logical) -> Logical;

    ////////////
    // supp.c
    pub static mut coutput: c_int;

}

#[repr(C)]
pub struct PrsVec {
    pub prsa: c_int,
    pub prsi: c_int,
    pub prso: c_int,
    pub prswon: Logical,
    pub prscon: c_int,
}
