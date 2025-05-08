#![cfg_attr(not(test), no_std, no_main)]

use driver::shell::shell;

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    poprint!("PANIC!");
    loop {}
}

pub mod polib;
pub mod driver;
pub mod apps;
pub mod fs;


#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    pomain();
    loop {}
}

pub fn pomain() {
    shell();
}

