KERNEL_BIN	 	= target/riscv64gc-unknown-none-elf/release/os
KERNEL_ENTRY_PA = 0x80000000

build:
	cargo build --release

test:
	cargo build --release
	qemu-system-riscv64 \
		-machine virt \
		-nographic \
		-bios none \
		-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA) \
		-s -S

gdb:
	riscv64-unknown-elf-gdb \
	-ex 'file $(KERNEL_BIN)' \
	-ex 'set arch riscv:rv64' \
	-ex 'target remote localhost:1234' \
	-ex 'set riscv use-compressed-breakpoints yes' \

user:
	cd ../user && make

auto: user
	make test

_asm:
	rust-objdump -S target/riscv64gc-unknown-none-elf/debug/os > os.tmp 2>/dev/null
