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
                                                    // we probably need to enable uart fifo as per https://www.youtube.com/watch?v=HC7b1SVXoKM

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

fn write_c(c: u8) {
    unsafe {
        while (*UART_LSR & UART_LSR_EMPTY_MASK) == 0 {}
        *UART_THR = c;
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

    for i in 0..8 {
        let byte: u8 = (id >> i) as u8;
        if byte == 0 {
            write_c(b"0"[0]);
        } else {
            write_c(byte);
        }
    }
    loop { }
}
