#![no_std]
#![no_main]

// The following info is specific to the Qemu virt machine.
// The base address is 0x80000000, the UART address base is 0x10000000
// The UART is UART16550
// https://opensocdebug.readthedocs.io/en/latest/02_spec/07_modules/dem_uart/uartspec.html

struct Uart {
    base: usize,
    thr: *mut u8,
    lsr: *mut u8,
    lsr_empty_mask: u8,
}

use core::fmt::Write;

impl Uart {
    fn new(base: usize, lsr_offset: usize, lsr_empty_mask: u8) -> Self {
        Self {
            base,
            thr: base as *mut u8,
            lsr: (base + lsr_offset) as *mut u8,
            lsr_empty_mask,
        }
    }
}

impl Write for Uart {
    #[inline(never)]
    #[unsafe(no_mangle)]
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        let lock = LOCK.lock();
        for byte in s.bytes() {
            unsafe {
                // Wait until the UART transmitter is empty
                while (core::ptr::read_volatile(self.lsr) & self.lsr_empty_mask) == 0 {}
                core::ptr::write_volatile(self.thr, byte);
            }
        }
        // Unlock lock
        drop(lock);
        Ok(())
    }
}

use core::sync::atomic::{AtomicBool, Ordering};

pub struct Spinlock {
    locked: AtomicBool,
}

impl Spinlock {
    pub const fn new() -> Self {
        Spinlock {
            locked: AtomicBool::new(false),
        }
    }

    #[inline(never)]
    #[unsafe(no_mangle)]
    pub fn lock(&self) -> SpinlockGuard {
        loop {
            match self
                .locked
                .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            {
                Ok(_) => break,
                Err(_) => continue,
            }
        }
        SpinlockGuard { lock: self }
    }
}

pub struct SpinlockGuard<'a> {
    lock: &'a Spinlock,
}

impl<'a> Drop for SpinlockGuard<'a> {
    #[unsafe(no_mangle)]
    fn drop(&mut self) {
        self.lock.locked.store(false, Ordering::Release);
    }
}

unsafe impl Sync for Spinlock {}

static LOCK: Spinlock = Spinlock::new();
static LOCK_2: Spinlock = Spinlock::new();

extern crate panic_halt;

use riscv_rt::entry;

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
            if  check_flag() {
                break;
            }
        }
        false
    }
}

#[entry]
fn main(hartid: usize) -> ! {
    if hartid == 0 {
        // Waking hart 1...
        set_flag();
    }
    let id = hartid;
    let mut uart = Uart::new(0x10000000, 5, 0x20);

    for _i in 0..10 {
        let _guard = LOCK_2.lock();

        if id == 0 {
            let _ = uart.write_str("I am: 0\n");
        } else {
            let _ = uart.write_str("I'm: 1\n");
        }
        let _ = writeln!(uart, "THIS IS WRITE from {}!", id);
    }
    loop {}
}
