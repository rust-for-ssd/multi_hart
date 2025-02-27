debug_target := target/riscv64gc-unknown-none-elf/debug/multi_hart_queue

run:
	cargo clean
	cargo run

build:
	cargo clean
	cargo build

release:
	cargo clean
	cargo build --release

gdb_server:
	qemu-system-riscv64 -smp 2 -machine virt -s -S -serial 'mon:stdio' -nographic -bios $(debug_target)

gdb_client:
	rust-gdb $(debug_target) -ex "target remote :1234" -ex "set print pretty on"
