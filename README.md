# OPSYS

A toy RISC-V microkernel operating system written in Rust.

## Feature

- Microkernel, relatively tiny kernel
- Mini shell, and various [user libraries](docs/userlib.md)
- Unix-like [system call interface](docs/syscall.md)
- Fast and opaque [IPC design](docs/ipc.md)

## How to run

```bash
cd os
make auto
```

Then, using gdb (in VScode/Terminal), you can see the output of the kernel. You may use `make gdb` at `os/` to run gdb.
