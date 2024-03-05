test:
	cargo build --release
	qemu-system-riscv64 \
	-machine virt \
	-nographic \
	-bios default \
	-kernel target/riscv64gc-unknown-none-elf/release/os \
	-s -S

gdb:
	riscv64-unknown-elf-gdb \
	-ex 'file target/riscv64gc-unknown-none-elf/release/os' \
	-ex 'set arch riscv:rv64' \
	-ex 'target remote localhost:1234'
