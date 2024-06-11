#![no_std]

use embedded_hal::blocking::spi::Transfer;
use embedded_hal::digital::v2::OutputPin;

// Function to read a register from the display controller
fn read_register<SPI, CS>(spi: &mut SPI, cs: &mut CS, command: u8, num_bytes: usize) -> Result<[u8; 3], SPI::Error>
    where
        SPI: Transfer<u8>,
        CS: OutputPin,
{
    let mut buffer = [0; 4]; // command byte + num_bytes
    buffer[0] = command;

    cs.set_low().ok();
    spi.transfer(&mut buffer)?;
    cs.set_high().ok();

    Ok([buffer[1], buffer[2], buffer[3]])
}

// Function to identify the ILI9341 or ST7789V
pub fn identify_display<SPI, CS>(spi: &mut SPI, cs: &mut CS) -> Result<&'static str, SPI::Error>
    where
        SPI: Transfer<u8>,
        CS: OutputPin,
{
    // Try to read ILI9341 ID (0x04)
    let ili9341_id = read_register(spi, cs, 0x04, 3)?;

    // Check if the ID matches ILI9341 (example values, actual values should be checked from datasheet)
    if ili9341_id == [0x93, 0x41, 0x00] {
        return Ok("ILI9341");
    }

    // Try to read ST7789V ID1 (0xDA), ID2 (0xDB), and ID3 (0xDC)
    let st7789v_id1 = read_register(spi, cs, 0xDA, 1)?;
    let st7789v_id2 = read_register(spi, cs, 0xDB, 1)?;
    let st7789v_id3 = read_register(spi, cs, 0xDC, 1)?;

    // Check if the ID matches ST7789V (example values, actual values should be checked from datasheet)
    if st7789v_id1[0] == 0x85 && st7789v_id2[0] == 0x85 && st7789v_id3[0] == 0x52 {
        return Ok("ST7789V");
    }

    Ok("Unknown")
}
