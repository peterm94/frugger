#![no_std]
#![no_main]

use bsp::entry;
use bsp::hal::{
    clocks::{Clock, init_clocks_and_plls},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};
use defmt::*;
#[allow(unused_imports)]
use defmt::*;
#[allow(unused_imports)]
use defmt_rtt as _f;
use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::spi::MODE_0;
use fugit::RateExtU32;
use input_test::InputTest;
use mipidsi::{Builder, Orientation};
#[allow(unused_imports)]
use panic_probe as _;
// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use waveshare_rp2040_zero as bsp;
use waveshare_rp2040_zero::{Gp0Spi0Rx, Gp1Spi0Csn, Gp2Spi0Sck, Gp3Spi0Tx};
use waveshare_rp2040_zero::hal::Timer;

use frugger_core::{ButtonInput, FruggerGame, FrugInputs};

use crate::mc_inputs::McInputs;

mod mc_inputs;

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

    let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );


    let mut led_pin = pins.gp5.into_push_pull_output();

    let left_pin = pins.gp15.into_pull_up_input();
    let left = left_pin.as_input();
    let right_pin = pins.gp14.into_pull_up_input();
    let right = right_pin.as_input();

    let up_pin = pins.gp27.into_pull_up_input();
    let up = up_pin.as_input();
    let down_pin = pins.gp26.into_pull_up_input();
    let down = down_pin.as_input();

    let a_pin = pins.gp7.into_pull_up_input();
    let a = a_pin.as_input();
    let b_pin = pins.gp8.into_pull_up_input();
    let b = b_pin.as_input();

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
    let spi = spi.init(&mut pac.RESETS, clocks.peripheral_clock.freq(), 20.MHz(), MODE_0);

    let di = SPIInterfaceNoCS::new(spi, dc);

    let mut display = Builder::ili9341_rgb565(di)
        .with_display_size(320, 240)
        .with_orientation(Orientation::Landscape(true))
        .init(&mut delay, Some(rst)).unwrap();

    delay.delay_ms(10);

    display.clear(Rgb565::CSS_ROYAL_BLUE).unwrap();

    // let mut game = BrickBreaker::new();
    let mut game = InputTest::new();
    // let mut game = Fire::new();

    let mut mc_inputs = McInputs::new(a, b, up, down, left, right);
    let mut frug_inputs = FrugInputs::default();

    const FRAME_TIME: u64 = 1000 / 10;

    loop {
        let frame_start = timer.get_counter();

        mc_inputs.tick(&mut frug_inputs);

        game.update(&frug_inputs);
        game.frugger().draw_frame(&mut display);

        let frame_end = timer.get_counter();
        let frame_elapsed = (frame_end - frame_start).to_millis();
        if frame_elapsed < FRAME_TIME {
            delay.delay_ms((FRAME_TIME - frame_elapsed) as u32);
        }
    }
}