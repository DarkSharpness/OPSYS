# User Library

We implement a user library to provide a set of functions that can be used by the user to interact with the system.

## IPC

We make a wrapping for the IPC system call to make it safer.

## File System

We make a wrapping for the standard input/output system calls. To ensure better performance, we implement a buffer for the standard input/output system calls (just as std::cin and std::cout in C++).

## Memory Management

We provide a user mode malloc and free function to manage memory in user mode.

## Others

We provide some tradition unix-like system calls, such as `exit`, `fork`, `exec`, `wait`. Still, we make more wrappings for the return value of the system calls to make it more user-friendly.
