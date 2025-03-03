#![no_std]
#![no_main]

use core::{
    cell::RefCell,
    sync::atomic::{AtomicBool, Ordering},
};
use critical_section::Mutex;
use qemu_uart::{UART, csprintln};
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

use heapless::Deque;
use multi_hart_critical_section as _;

static QUEUE1: Mutex<RefCell<Deque<usize, 50>>> =
    Mutex::new(RefCell::new(Deque::<usize, 50>::new()));

static QUEUE2: Mutex<RefCell<Deque<usize, 50>>> =
Mutex::new(RefCell::new(Deque::<usize, 50>::new()));

#[entry]
fn main(hartid: usize) -> ! {
    if hartid == 0 {
        set_flag();
        
        // HART 0 adds to the first queue
        for i in 0..10 {
            critical_section::with(|cs| {
                let mut queue = QUEUE1.borrow_ref_mut(cs);
                let _ = queue.push_back(i);
                csprintln!(cs, "Hart 0 added: {}, Queue: {:?}", i, queue);
            });
        }
    } else {
        // Everyone else reads from queue 1 and adds to queue 2
        for _ in 0..20 {
            critical_section::with(|cs| {
                let mut queue = QUEUE1.borrow_ref_mut(cs);
                if let Some(value) = queue.pop_front() {
                    csprintln!(cs, "Hart {} read: {}", hartid, value);
                    let mut queue = QUEUE2.borrow_ref_mut(cs);
                    let _ = queue.push_back(value);
                }
                
            });
        }
    }

    if hartid == 0 {
      critical_section::with(|cs| {
        let mut queue1 = QUEUE1.borrow_ref_mut(cs);
        let mut queue2 = QUEUE2.borrow_ref_mut(cs);
        csprintln!(cs, "---Final queues ---");
        csprintln!(cs, "Queue1: {:?}", queue1);
        csprintln!(cs, "Queue2: {:?}", queue2);

    });  
    }
    //print final queues
    


    loop {}
}
