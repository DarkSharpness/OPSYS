# Syscall Interface

To support traditional unix system calls, we still provide these syscalls. However, in theory, all these syscalls can be implemented with only simple ipc syscalls (request / receive / respond).

Currently supported syscalls:

- fork
- exec
- exit
- wait
- close
- read
- write
- yield
- request
- receive
- respond
- sbrk

For the core of those syscalls, please refer to [ipc docs](ipc.md).
