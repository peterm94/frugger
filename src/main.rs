#![no_std]
#![no_main]

use bsp::entry;
use defmt::*;

#[allow(unused_imports)]
use defmt::*;
#[allow(unused_imports)]
use defmt_rtt as _f;
#[allow(unused_imports)]
use panic_probe as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use waveshare_rp2040_zero as bsp;
use waveshare_rp2040_zero::{Gp0Spi0Rx, Gp1Spi0Csn, Gp2Spi0Sck, Gp3Spi0Tx};

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};
use display_interface_spi::{SPIInterfaceNoCS};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle, StyledDrawable};
use embedded_hal::digital::OutputPin;
use embedded_hal::spi::MODE_0;
use fugit::RateExtU32;
use mipidsi::Builder;
use mipidsi::options::TearingEffect;

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );


    let mut led_pin = pins.gp5.into_push_pull_output();

    // turn on the backlight
    led_pin.set_high().unwrap();

    let mut rst = pins.gp6.into_push_pull_output();
    rst.set_high().unwrap();

    let dc = pins.gp4.into_push_pull_output();

    let rx: Gp0Spi0Rx = pins.gp0.reconfigure();
    let tx: Gp3Spi0Tx = pins.gp3.reconfigure();
    let _cs: Gp1Spi0Csn = pins.gp1.reconfigure();
    let sck: Gp2Spi0Sck = pins.gp2.reconfigure();

    let spi: bsp::hal::spi::Spi::<_, _, _, 8> = bsp::hal::spi::Spi::new(pac.SPI0, (tx, rx, sck));
    let spi = spi.init(&mut pac.RESETS, clocks.peripheral_clock.freq(), 12000.kHz(), MODE_0);

    let di = SPIInterfaceNoCS::new(spi, dc);

    let mut display = Builder::ili9341_rgb565(di)
        .with_display_size(320, 240)
        .init(&mut delay, Some(rst)).unwrap();


    display.set_tearing_effect(TearingEffect::HorizontalAndVertical).unwrap();
    let rect = Rectangle::with_corners(Point::new(0, 0), Point::new(200, 200));

    let mut pos = 0;

    display.clear(Rgb565::CSS_INDIAN_RED).unwrap();

    loop {
        for j in 0..10 {
            for k in 0..10 {
                display.set_pixel(pos + j, pos + k, Rgb565::CSS_INDIAN_RED).unwrap();

            }
        }

        pos += 1;

        for j in 0..10 {
            for k in 0..10 {
                display.set_pixel(pos + j, pos + k, Rgb565::CSS_GOLD).unwrap();

            }
        }

        delay.delay_ms(16);

        // display.clear(Rgb565::CSS_INDIAN_RED).unwrap();
        // display.clear(Rgb565::new(r as u8, g as u8, b as u8)).unwrap();
        // display.clear(Rgb565::CSS_INDIAN_RED).unwrap();
        // display.clear(Rgb565::CSS_DARK_GOLDENROD).unwrap();
        // display.clear(Rgb565::CSS_ROYAL_BLUE).unwrap();
        // display.clear(Rgb565::CSS_GOLD).unwrap();
        // display.clear(Rgb565::CSS_INDIGO).unwrap();
    }
}