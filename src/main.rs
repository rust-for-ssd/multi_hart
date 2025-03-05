#![no_std]
#![no_main]

extern crate panic_halt;

extern crate alloc;


use alloc::vec::Vec;
use core::{
    cell::RefCell,
    sync::atomic::{AtomicBool, Ordering},
};
use qemu_uart::{UART, csprintln};
use riscv_rt::entry;
use embedded_alloc::LlffHeap as Heap;

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
        const HEAP_SIZE: usize = 1024;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(&raw mut HEAP_MEM as usize, HEAP_SIZE) }
        critical_section::with(|cs| {
            csprintln!(cs, "Mem arr {:p}", {&raw mut HEAP_MEM})
        });   
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

use multi_hart_critical_section as _;
use core::mem::MaybeUninit;


#[global_allocator]
static HEAP: Heap = Heap::empty();



#[entry]
fn main(hartid: usize) -> ! {
    if hartid == 0 {
        
        

        set_flag();

    }

    let mut xs = Vec::new();

    critical_section::with(|cs| {
        xs.push(hartid);
        csprintln!(cs, "Hart {} added to shared list", hartid);
    });



    loop {}
}
