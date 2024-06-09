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
use display_interface::WriteOnlyDataCommand;
use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle};
use embedded_graphics::primitives::rectangle::StyledPixelsIterator;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::spi::MODE_0;
use fugit::RateExtU32;
use mipidsi::{Builder, Display, Orientation};
use mipidsi::models::Model;
#[allow(unused_imports)]
use panic_probe as _;
// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use waveshare_rp2040_zero as bsp;
use waveshare_rp2040_zero::{Gp0Spi0Rx, Gp1Spi0Csn, Gp2Spi0Sck, Gp3Spi0Tx};

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
        .with_orientation(Orientation::Landscape(false))
        .init(&mut delay, Some(rst)).unwrap();

    delay.delay_ms(10);

    // display.set_tearing_effect(TearingEffect::HorizontalAndVertical).unwrap();

    let style = PrimitiveStyleBuilder::new()
        .stroke_color(FruggerColour::Red)
        .stroke_width(3)
        .fill_color(FruggerColour::Green)
        .build();
    let mut rec2 = Rectangle::new(Point::new(0, 0), Size::new(10, 10))
        .into_styled(style);

    display.clear(Rgb565::CSS_CHOCOLATE).unwrap();

    let style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);

    let mut frugger = Frugger::new(display);
    // let mut now_disp: String<4> = String::new();
    // core.1
    // let mut syst = core2.SYST;
    // syst.enable_counter();
    let now = "banana";

    let mut frame_time = 0;
    let mut x = false;
    loop {
        frugger.draw(rec2.pixels());
        frugger.draw_frame();
    }
}

const COLOURS: [Rgb565; 8] = [Rgb565::RED, Rgb565::GREEN, Rgb565::CSS_ROYAL_BLUE, Rgb565::BLACK, Rgb565::WHITE, Rgb565::CSS_ORANGE, Rgb565::CSS_PURPLE, Rgb565::CSS_LIGHT_GOLDENROD_YELLOW];

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
enum FruggerColour {
    Black,
    White,
    Green,
    Blue,
    Red,
    Yellow,
    Purple,
    Orange,
}

impl FruggerColour {
    fn idx(index: u8) -> Self {
        match index {
            0 => FruggerColour::Black,
            1 => FruggerColour::White,
            2 => FruggerColour::Green,
            3 => FruggerColour::Blue,
            4 => FruggerColour::Red,
            5 => FruggerColour::Yellow,
            6 => FruggerColour::Purple,
            7 => FruggerColour::Orange,
            _ => FruggerColour::Black
        }
    }

    fn bits(&self) -> u8 {
        match self {
            FruggerColour::Black => { 0 }
            FruggerColour::White => { 1 }
            FruggerColour::Green => { 2 }
            FruggerColour::Blue => { 3 }
            FruggerColour::Red => { 4 }
            FruggerColour::Yellow => { 5 }
            FruggerColour::Purple => { 6 }
            FruggerColour::Orange => { 7 }
        }
    }

    fn rgb565(&self) -> Rgb565 {
        match self {
            FruggerColour::Black => Rgb565::BLACK,
            FruggerColour::White => Rgb565::WHITE,
            FruggerColour::Green => Rgb565::GREEN,
            FruggerColour::Blue => Rgb565::BLUE,
            FruggerColour::Red => Rgb565::RED,
            FruggerColour::Yellow => Rgb565::YELLOW,
            FruggerColour::Purple => Rgb565::CSS_PURPLE,
            FruggerColour::Orange => Rgb565::CSS_ORANGE
        }
    }
}

impl PixelColor for FruggerColour {
    type Raw = ();
}

struct Frugger<DI, MODEL, RST> where DI: WriteOnlyDataCommand,
                                     MODEL: Model<ColorFormat = Rgb565>,
                                     RST: OutputPin {
    // 320 * 240 * (3 bits per color)
    last_frame: [u8; 28800],
    next_frame: [u8; 28800],
    display: Display<DI, MODEL, RST>,
}

impl<DI, MODEL, RST> Frugger<DI, MODEL, RST> where
    DI: WriteOnlyDataCommand,
    MODEL: Model<ColorFormat = Rgb565>,
    RST: OutputPin {
    fn new(display: Display<DI, MODEL, RST>) -> Self {
        Self {
            last_frame: [0u8; 28800],
            next_frame: [0u8; 28800],
            display,
        }
    }
    fn get_pixel_value(&self, x: u16, y: u16) -> FruggerColour {
        let pixel_offset = y * 320 + x;
        let bit_index = pixel_offset * 3;
        let byte_index = (bit_index / 8) as usize;
        let bit_offset = bit_index % 8;

        if bit_offset <= 5 {
            FruggerColour::idx((self.last_frame[byte_index] >> bit_index) & 0b111)
        } else {
            let p1 = self.last_frame[byte_index] >> bit_offset;
            let p2 = self.last_frame[byte_index + 1] << (8 - bit_offset);
            FruggerColour::idx((p1 | p2) & 0b111)
        }
    }

    fn get_pixel_value_next(&self, x: u16, y: u16) -> FruggerColour {
        let pixel_offset = y * 320 + x;
        let bit_index = pixel_offset * 3;
        let byte_index = (bit_index / 8) as usize;
        let bit_offset = bit_index % 8;

        if bit_offset <= 5 {
            FruggerColour::idx((self.next_frame[byte_index] >> bit_index) & 0b111)
        } else {
            let p1 = self.next_frame[byte_index] >> bit_offset;
            let p2 = self.next_frame[byte_index + 1] << (8 - bit_offset);
            FruggerColour::idx((p1 | p2) & 0b111)
        }
    }

    fn write_pixel_value(&mut self, x: u16, y: u16, colour: FruggerColour) {
        let pixel_offset = y * 320 + x;
        let bit_index = pixel_offset * 3;
        let byte_index = (bit_index / 8) as usize;
        let bit_offset = bit_index % 8;
        let bits = colour.bits();

        if bit_offset <= 5 {
            self.next_frame[byte_index] &= !(0b111 << bit_offset);
            self.next_frame[byte_index] |= (bits & 0b111) << bit_offset;
        } else {
            let p1 = bits & (0b111 >> (bit_offset - 5));
            let p2 = bits >> (8 - bit_offset);

            self.next_frame[byte_index] &= !(0b111 << bit_offset);
            self.next_frame[byte_index] |= p1 << bit_offset;
            self.next_frame[byte_index + 1] &= !(0b111 >> (8 - bit_offset));
            self.next_frame[byte_index + 1] |= p1;
        }
    }

    fn draw(&mut self, pixels: StyledPixelsIterator<FruggerColour>) {
        for Pixel(point, col) in pixels {
            self.write_pixel_value(point.x as u16, point.y as u16, col);
        }
    }
    fn draw_frame(&mut self)
    {
        for x in 0..320 {
            for y in 0..240 {
                let next = self.get_pixel_value_next(x, y);
                if next != self.get_pixel_value(x, y) {
                    self.display.set_pixel(x, y, next.rgb565()).unwrap()
                }
            }
        }

        // Override last frame with the new one
        self.last_frame.copy_from_slice(&self.next_frame);
    }
}