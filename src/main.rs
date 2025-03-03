#![no_std]
#![no_main]

use core::{
    cell::RefCell,
    sync::atomic::{AtomicBool, Ordering},
};
use critical_section::Mutex;
use riscv_rt::entry;
use semihosting::{
    println,
    process::{ExitCode, exit},
};

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
    for i in 0..=50u32.div_ceil(4) {
        critical_section::with(|cs| {
            let mut dq = DEQUE.borrow_ref_mut(cs);
            let _ = dq.push_front(hartid);
            println!("HID: {}, i: {}, {:?}", hartid, i, dq);
        });
    }

    // ExitCode::SUCCESS.exit_process();
    loop {}
}

#[unsafe(export_name = "DefaultHandler")]
unsafe fn custom_interrupt_handler() {
    println!("THIS IS FROM DEFAULT");
    loop {}
}

#[unsafe(export_name = "ExceptionHandler")]
unsafe fn custom_exception_handler() {
    println!(
        "THIS IS FROM EXCEPTION: {}",
        riscv::register::mhartid::read()
    );
    let mepc = riscv::register::mepc::read();
    println!("{}", mepc);
    loop {}
}
