OUTPUT_ARCH(riscv)
ENTRY(os_start)
BASE_ADDRESS = 0x80000000;

SECTIONS
{
    . = BASE_ADDRESS;
    skernel = .;
    stext = .;
    .text : {
        *(.text.entry)
    }

    . = BASE_ADDRESS + 0x2000;
    .data : {
        *(.data.pagetable)
    }

    text = .;
    .text : {
        *(.text .text.*)
    }

    . = ALIGN(4K);
    etext = .;
    srodata = .;
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }

    . = ALIGN(4K);
    erodata = .;
    sdata = .;
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }

    . = ALIGN(4K);
    edata = .;
    sbss_real = .;
    .bss : {
        *(.bss.stack)
        sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }

    . = ALIGN(4K);
    ebss = .;
    ekernel = .;

    /DISCARD/ : {
        *(.eh_frame)
    }
}