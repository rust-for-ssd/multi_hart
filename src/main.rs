#![no_std]
#![no_main]

use core::{
    cell::RefCell,
    sync::atomic::{AtomicBool, Ordering},
};
use critical_section::Mutex;
use qemu_uart::{UART, csprintln};
use riscv_rt::entry;
// use semihosting::println;
extern crate panic_halt;

static INIT_PROGRAM_FLAG: AtomicBool = AtomicBool::new(true);

fn set_flag() {
    INIT_PROGRAM_FLAG.store(true, Ordering::SeqCst);
}

fn check_flag() -> bool {
    INIT_PROGRAM_FLAG.load(Ordering::SeqCst)
}

#[unsafe(export_name = "_mp_hook")]
#[rustfmt::skip]
pub extern "Rust" fn user_mp_hook(hartid: usize) -> bool {
    if hartid == 0 {
        true
    } else {
        loop {
            if check_flag() {
                break;
            }
        }
        false
    }
}

use heapless::Deque;
use multi_hart_critical_section as _;

static DEQUE: Mutex<RefCell<Deque<usize, 50>>> =
    Mutex::new(RefCell::new(Deque::<usize, 50>::new()));

#[entry]
fn main(hartid: usize) -> ! {
    if hartid == 0 {
        set_flag();
    }
    for _i in 0..=50u32.div_ceil(4) {
        critical_section::with(|cs| {
            let mut dq = DEQUE.borrow_ref_mut(cs);
            let _ = dq.push_front(hartid);
            csprintln!(cs, "{:?}", dq);
        });
    }

    // ExitCode::SUCCESS.exit_process();
    loop {}
}
