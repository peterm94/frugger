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
use embedded_graphics::mono_font::ascii::{FONT_10X20, FONT_6X10};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle};
use embedded_graphics::text::Text;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::spi::MODE_0;
use fugit::{Rate, RateExtU32};
use heapless::String;
use mipidsi::{Builder, Orientation};
#[allow(unused_imports)]
use panic_probe as _;
// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use waveshare_rp2040_zero as bsp;
use waveshare_rp2040_zero::{Gp0Spi0Rx, Gp1Spi0Csn, Gp2Spi0Sck, Gp3Spi0Tx};

use frugger_core::{Frugger, FruggerColour};

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
    let spi = spi.init(&mut pac.RESETS, clocks.peripheral_clock.freq(), 20.MHz(), MODE_0);

    let di = SPIInterfaceNoCS::new(spi, dc);

    let mut display = Builder::ili9341_rgb565(di)
        .with_display_size(320, 240)
        .with_orientation(Orientation::Landscape(true))
        .init(&mut delay, Some(rst)).unwrap();

    delay.delay_ms(10);

    // display.set_tearing_effect(TearingEffect::HorizontalAndVertical).unwrap();

    let f_style = PrimitiveStyleBuilder::new()
        .stroke_color(FruggerColour::Red)
        .stroke_width(1)
        .fill_color(FruggerColour::Green)
        .build();
    let f_style2 = PrimitiveStyleBuilder::new()
        .stroke_color(FruggerColour::Purple)
        .stroke_width(1)
        .fill_color(FruggerColour::Orange)
        .build();


    let style = PrimitiveStyleBuilder::new()
        .stroke_color(Rgb565::WHITE)
        .stroke_width(3)
        .fill_color(Rgb565::GREEN)
        .build();

    let style2 = PrimitiveStyleBuilder::new()
        .stroke_color(Rgb565::RED)
        .stroke_width(3)
        .fill_color(Rgb565::YELLOW)
        .build();

    display.clear(Rgb565::YELLOW).unwrap();
    display.clear(Rgb565::CSS_ROYAL_BLUE).unwrap();

    let text_style = MonoTextStyle::new(&FONT_6X10, FruggerColour::White);

    let mut frugger = Frugger::new(FruggerColour::Purple, &mut display);

    let mut x = 0;

    let mut loc_text: String<4> = String::new();

    let oofs = [[0, 10], [50, 100], [20, 0], [120, 40], [200, 21], [300, 179], [142, 65]];

    loop {
        x = (x + 1) % 320;

        for oof in oofs {
            let rec2 = Rectangle::new(Point::new((oof[0] + x) % 320, oof[1]), Size::new(50, 50))
                .into_styled(f_style);
            rec2.draw(&mut frugger);
        }

        loc_text.clear();
        let y = String::<4>::try_from(x).unwrap();
        loc_text.push_str(y.as_str());

        Text::new(loc_text.as_str(), Point::new(30 + x, 30), text_style).draw(&mut frugger);

        frugger.draw_frame();
    }
}