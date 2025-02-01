use crate::mc_inputs::McInputs;
use core::cell::RefCell;
use fugit::RateExtU32;
use sh1106::prelude::*;
use sh1106::Builder;

use bsp::hal::clocks::SystemClock;
use bsp::hal::{Sio, Timer};
use bsp::pac;
use embedded_graphics::Pixel;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::DrawTarget;
use frugger_core::{ButtonInput, FrugDisplay, FrugTimer, Orientation};
use frugger_onebit::{OneBitDisplay, OneBitRunner};
use sh1106::interface::DisplayInterface;
use waveshare_rp2040_zero as bsp;

#[link_section = ".data_storage"]
static mut DATA_STORAGE: [u8; 1024] = [0u8; 1024];

struct HalTimer(Timer);

impl FrugTimer for HalTimer {
    fn ticks(&self) -> u64 {
        self.0.get_counter().ticks()
    }

    fn delay_ms(&mut self, ms: u64) {
        embedded_hal::delay::DelayNs::delay_ms(&mut self.0, ms as u32);
    }
}

struct SmallDisplay<DI>(GraphicsMode<DI>)
where
    DI: DisplayInterface;

// TODO not like this
impl<DI> DrawTarget<Color = BinaryColor> for SmallDisplay<DI>
where
    DI: DisplayInterface,{
    type Color = ;
    type Error = ();

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item=Pixel<Self::Color>>
    {
        todo!()
    }
}
impl<DI> FrugDisplay for SmallDisplay<DI>
where
    DI: DisplayInterface,
{
    fn flush(&mut self) {
        self.0.flush();
    }
    fn set_orientation(&mut self, orientation: &Orientation) {
        match orientation {
            Orientation::Landscape => {
                self.0.set_rotation(DisplayRotation::Rotate0);
            }
            Orientation::Portrait => {
                self.0.set_rotation(DisplayRotation::Rotate90);
            }
        }
    }
}

pub(crate) fn start(system_clock: &SystemClock, mut timer: Timer) -> ! {
    // I don't know if I like this, but it seems necessary(?)
    let mut pac = unsafe { pac::Peripherals::steal() };

    let sio = Sio::new(pac.SIO);

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Set up inputs
    let left_pin = pins.gp15.into_pull_up_input();
    let left = left_pin.as_input();
    let right_pin = pins.gp14.into_pull_up_input();
    let right = right_pin.as_input();

    // let up_pin = pins.gp27.into_pull_up_input();
    let up_pin = pins.gp3.into_pull_up_input();
    let up = up_pin.as_input();
    let down_pin = pins.gp26.into_pull_up_input();
    let down = down_pin.as_input();

    // let a_pin = pins.gp7.into_pull_up_input();
    let a_pin = pins.gp2.into_pull_up_input();
    let a = a_pin.as_input();
    // let b_pin = pins.gp8.into_pull_up_input();
    let b_pin = pins.gp4.into_pull_up_input();
    let b = b_pin.as_input();

    let mut hw_inputs = RefCell::new(McInputs::new(a, b, up, down, left, right));

    // Set up screen
    let sda_pin = pins.gp0.reconfigure();
    let scl_pin = pins.gp1.reconfigure();

    let i2c = bsp::hal::i2c::I2C::i2c0(
        pac.I2C0,
        sda_pin,
        scl_pin,
        1.MHz(),
        &mut pac.RESETS,
        system_clock,
    );
    let mut display: GraphicsMode<_> = Builder::new().connect_i2c(i2c).into();
    display.init().unwrap();
    display.flush().unwrap();

    let mut display = OneBitDisplay(SmallDisplay(display));

    let mut timer = HalTimer(timer);

    let mut runner = OneBitRunner::new(
        unsafe { DATA_STORAGE.as_mut_ptr() },
        display,
        |input| hw_inputs.borrow_mut().tick(input),
        &mut timer,
    );

    runner.start();
}
