param (
    [string]$arg
)

# How to make this work
# Get picotool. Make sure it is on the path.
# Get Zadig
# Use Zadig to install WinUSB for the device. The device should match the name in the "product" of the USB device.
# Just run cargo run and it will work

# Reboot the device into BOOTSEL using picotool
& picotool reboot -f -u
Start-Sleep -Seconds 1

# Run elf2uf2-rs to upload it to the board
& elf2uf2-rs -d $arg
