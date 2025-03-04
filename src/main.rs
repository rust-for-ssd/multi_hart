#![no_std]
#![no_main]

use core::{
    sync::atomic::{AtomicBool, Ordering},
};
use qemu_uart::unsafeprintln;
use riscv_rt::entry;
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

use heapless::mpmc::Q64;

static QUEUE: Q64<u8> = Q64::new();


#[entry]
fn main(hartid: usize) -> ! {

    if hartid == 0 {
        
        
        // HART 0 adds to the first queue
        for i in 0..10 {
            QUEUE.enqueue(i).ok();
            unsafeprintln!("HART {} added {} Queue Pointer: {:p}", hartid , i, &QUEUE);

        }
        
        set_flag();
    }    

    // Everyone reads from queue
    for _ in 0..20 {
        if let Some(x) = QUEUE.dequeue() {
            unsafeprintln!("HART {} read {}", hartid , x)
            
        } else { 
            // unsafeprintln!("Something went wrong");
        }
    }


    loop {}
}
