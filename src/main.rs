#![no_std]
#![no_main]

// The following info is specific to the Qemu virt machine.
// The base address is 0x80000000, the UART address base is 0x10000000
// The UART is UART16550
// https://opensocdebug.readthedocs.io/en/latest/02_spec/07_modules/dem_uart/uartspec.html
const UART_BASE: usize = 0x10000000;
const UART_THR: *mut u8 = UART_BASE as *mut u8;     // Transmit Holding Register
const UART_LSR: *mut u8 = (UART_BASE + 5) as *mut u8; // Line Status Register
const UART_LSR_EMPTY_MASK: u8 = 0x20;               // Transmitter Empty bit

struct Uart {
    base: usize,
    thr: *mut u8,
    lsr: *mut u8,
    lsr_empty_mask: u8,
}

use core::fmt::Write;

impl Uart {
    fn new() -> Self {
        Self {
            base: UART_BASE,
            thr: UART_THR,
            lsr: UART_LSR,
            lsr_empty_mask: UART_LSR_EMPTY_MASK,
        }
    }
}

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        // take the lock
        //let _guard = LOCK.lock();
        for byte in s.bytes() {
            unsafe {
                // Wait until the UART transmitter is empty
                while (*self.lsr & self.lsr_empty_mask) == 0 {}
                *self.thr = byte;
            }
        }
        Ok(())
    }
}

use core::sync::atomic::{AtomicUsize, Ordering};

pub struct Spinlock {
    // Changed from AtomicBool to AtomicUsize.
    // 0 means unlocked, 1 means locked.
    locked: AtomicUsize,
}

impl Spinlock {
    pub const fn new() -> Self {
        Spinlock {
            locked: AtomicUsize::new(0),
        }
    }

    #[unsafe(no_mangle)]
    pub fn lock(&self) -> SpinlockGuard {
        loop {
            // Try to change from 0 (unlocked) to 1 (locked).
            match self.locked.compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed) {
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
        self.lock.locked.store(0, Ordering::Release);
    }
}

// This spinlock is now using a native word-sized atomic operation,
// so on riscv64gc the compare_exchange will compile into an AMO instruction.
unsafe impl Sync for Spinlock {}

static LOCK: Spinlock = Spinlock::new();
static LOCK_2: Spinlock = Spinlock::new();

extern crate panic_halt;

use riscv::asm::wfi;
use riscv::register::{mie, mip};
use riscv_rt::entry;

#[unsafe(export_name = "_mp_hook")]
#[rustfmt::skip]
pub extern "Rust" fn user_mp_hook(hartid: usize) -> bool {
    if hartid == 0 {
        true
    } else {
        let addr = 0x02000000 + hartid * 4;
        unsafe {
            // Clear IPI
            (addr as *mut u32).write_volatile(0);

            // Start listening for software interrupts
            mie::set_msoft();

            loop {
                wfi();
                if mip::read().msoft() {
                    break;
                }
            }

            // Stop listening for software interrupts
            mie::clear_msoft();

            // Clear IPI
            (addr as *mut u32).write_volatile(0);
        }
        false
    }
}

#[entry]
fn main(hartid: usize) -> ! {
    if hartid == 0 {
        // Waking hart 1...
        let addr = 0x02000004;
        unsafe {
            (addr as *mut u32).write_volatile(1);
        }
    }
    let id = hartid;
    {
        //let _guard = LOCK_2.lock();

    //write!(Uart::new(), "*{}*\n", id).ok();
        let mut uart = Uart::new();
        if id == 0 {
        uart.write_str("aaaaaaaaaaa");
        } else {
        uart.write_str("bbbbbbbbbbb");

        }

        write!(Uart::new(), "*{}*\n", id).ok();
        //drop(_guard);
    }
    loop { }
}
