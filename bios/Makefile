build:
	cargo build --release

display:
	cargo build --release
	rust-objcopy --strip-all target/riscv64gc-unknown-none-elf/release/bios \
		-O binary target/riscv64gc-unknown-none-elf/release/bios.bin
	clear
	stat target/riscv64gc-unknown-none-elf/release/bios.bin