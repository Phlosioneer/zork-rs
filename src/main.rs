
extern crate libc;

#[link(name = "c_zork")]
extern {
    fn c_main();
}


fn main() {
    println!("Hello, world!");

    unsafe {
        c_main();
    }
}
