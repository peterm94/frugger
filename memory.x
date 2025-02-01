MEMORY {
    BOOT2 : ORIGIN = 0x10000000, LENGTH = 0x100
    FLASH : ORIGIN = 0x10000100, LENGTH = 2048K - 0x100 - 0x400
    DATA  : ORIGIN = 0x10200000, LENGTH = 0x400
    RAM   : ORIGIN = 0x20000000, LENGTH = 256K
}

EXTERN(BOOT2_FIRMWARE)

SECTIONS {
    /* ### Boot loader */
    .boot2 ORIGIN(BOOT2) :
    {
        KEEP(*(.boot2));
    } > BOOT2

    /* ### Data storage */
    .data_storage ORIGIN(DATA) :
    {
        . = ALIGN(4);
        KEEP(*(.data_storage));
    } > DATA

} INSERT BEFORE .text;
