BIOS_PATH = ../bios/target/riscv64gc-unknown-none-elf/release/bios
OS_PATH = target/riscv64gc-unknown-none-elf/release/os

build:
	cargo build --release

test:
	cargo build --release
	qemu-system-riscv64 \
	-machine virt \
	-nographic \
	-bios $(BIOS_PATH) \
	-kernel $(OS_PATH) \
	-s -S

old:
	cargo build --release
	qemu-system-riscv64 \
	-machine virt \
	-nographic \
	-bios default \
	-kernel target/riscv64gc-unknown-none-elf/release/os \
	-s -S

gdb:
	riscv64-unknown-elf-gdb \
	-ex 'file $(OS_PATH)' \
	-ex 'set arch riscv:rv64' \
	-ex 'target remote localhost:1234'

asm:
	rust-objdump -S target/riscv64gc-unknown-none-elf/debug/os > os.tmp 2>/dev/null
