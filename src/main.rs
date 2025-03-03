#![no_std]
#![no_main]

use core::{
    cell::RefCell,
    sync::atomic::{AtomicBool, Ordering, compiler_fence, fence},
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

use embedded_alloc::LlffHeap as Heap;
#[global_allocator]
static HEAP: Heap = Heap::empty();

unsafe extern "C" {
    static _heap_size: u8;

}

extern crate alloc;
use alloc::collections::VecDeque;
static HEAP_QUEUE: Mutex<RefCell<Option<VecDeque<usize>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main(hartid: usize) -> ! {
    if hartid == 0 {
        unsafe {
            let heap_start = riscv_rt::heap_start() as usize;
            let heap_allign = (heap_start % 32) + heap_start + 32;

            let heap_size = &_heap_size as *const u8 as usize;
            HEAP.init(heap_allign, heap_size - (heap_start % 32))
        }

        critical_section::with(|cs| {
            *HEAP_QUEUE.borrow_ref_mut(cs) = Some(VecDeque::with_capacity(10));
        });
        compiler_fence(Ordering::SeqCst);
        set_flag();
    }
    for _i in 0..=50u32.div_ceil(4) {
        critical_section::with(|cs| {
            let mut dq = DEQUE.borrow_ref_mut(cs);
            let _ = dq.push_front(hartid);
            csprintln!(cs, "{:?}", dq);
            let heap_bottom = riscv_rt::heap_start() as usize;
            let heap_size = unsafe { &_heap_size } as *const u8 as usize;
            let heap_alligned = (heap_bottom % 32) + heap_bottom + 32;
            csprintln!(
                cs,
                "heapsize: {}, heapbottom: {}, heap_alligned: {}",
                heap_size,
                heap_bottom,
                heap_alligned
            );
            // Use the heap allocated queue.
            if hartid == 0 {
                if let Some(ref mut heap_q) = *HEAP_QUEUE.borrow_ref_mut(cs) {
                    let _ = heap_q.push_back(hartid);
                    csprintln!(cs, "Heap QUEUE: {:?}", heap_q);
                }
            }
        });
    }

    // ExitCode::SUCCESS.exit_process();
    loop {}
}
