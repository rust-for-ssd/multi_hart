#![no_std]
#![no_main]

use core::{
    sync::atomic::{AtomicBool, Ordering},
};
use qemu_uart::{unsafeprintln};
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

const UART_BASE: usize = 0x10000000;
const UART_THR: *mut u8 = UART_BASE as *mut u8;     // Transmit Holding Register
const UART_LSR: *mut u8 = (UART_BASE + 5) as *mut u8; // Line Status Register
const UART_LSR_EMPTY_MASK: u8 = 0x20;               // Transmitter Empty bit
                                                    // we probably need to enable uart fifo as per https://www.youtube.com/watch?v=HC7b1SVXoKM
fn write_c(c: u8) {
    unsafe {
        while (*UART_LSR & UART_LSR_EMPTY_MASK) == 0 {}
        *UART_THR = c;
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
            // uart.println("Hart 0 added: {}", i);

            unsafeprintln!("HART {} added {} Queue Pointer: {:p}", hartid , i, &QUEUE);


            
             

            
            // critical_section::with(|cs| {
            //     let mut queue = QUEUE1.borrow_ref_mut(cs);
            //     let _ = queue.push_back(i);
            //     csprintln!(cs, "Hart 0 added: {}, Queue: {:?}", i, queue);
            // });
        }

        // for i in 0..10 { 
        //     if let Some(x) = QUEUE.dequeue() { 
        //         unsafeprintln!("HART {} dequeued {}", hartid , x);
        //     } else {}
        // }
        set_flag();
    }
        // Everyone else reads from queue 1 and adds to queue 2
        for _ in 0..20 {
            if let Some(x) = QUEUE.dequeue() {
                // csprintln!("Hart {} read: {}", hartid, x);
                unsafeprintln!("HART {} read {}", hartid , x)
                
            } else { 
                // unsafeprintln!("Something went wrong");
                // unsafeprintln!("QUEUE pointer{:p}", &QUEUE)

            }
            // critical_section::with(|cs| {
            //     let mut queue = QUEUE1.borrow_ref_mut(cs);
            //     if let Some(value) = queue.pop_front() {
            //         csprintln!(cs, "Hart {} read: {}", hartid, value);
            //         let mut queue = QUEUE2.borrow_ref_mut(cs);
            //         let _ = queue.push_back(value);
            //     }
                
            // });
        }

    // if hartid == 0 {
    //   critical_section::with(|cs| {
    //     let mut queue1 = QUEUE1.borrow_ref_mut(cs);
    //     let mut queue2 = QUEUE2.borrow_ref_mut(cs);
    //     csprintln!(cs, "---Final queues ---");
    //     csprintln!(cs, "Queue1: {:?}", queue1);
    //     csprintln!(cs, "Queue2: {:?}", queue2);

    // });  
    // }
    //print final queues
    


    loop {}
}
