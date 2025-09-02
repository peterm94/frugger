use crate::mc_inputs::McInputs;
use fugit::RateExtU32;
use sh1106::prelude::*;
use sh1106::Builder;

use bsp::hal::clocks::SystemClock;
use bsp::hal::{Sio, Timer};
use bsp::pac;
use embedded_hal::delay::DelayNs;
use frugger_core::util::RollingAverage;
use frugger_core::{ButtonInput, FrugInputs, FrugTimer, FruggerEngine, FruggerGame};
use frugger_onebit::menu::Menu;
use sh1106::interface::DisplayInterface;
use ssd1306::prelude::DisplayConfig;
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
    let left_pin = pins.gp14.into_pull_up_input();
    let left = left_pin.as_input();
    let right_pin = pins.gp26.into_pull_up_input();
    let right = right_pin.as_input();

    let up_pin = pins.gp27.into_pull_up_input();
    // let up_pin = pins.gp3.into_pull_up_input();
    let up = up_pin.as_input();
    let down_pin = pins.gp2.into_pull_up_input();
    let down = down_pin.as_input();

    let a_pin = pins.gp15.into_pull_up_input();
    // let a_pin = pins.gp2.into_pull_up_input();
    let a = a_pin.as_input();
    let b_pin = pins.gp8.into_pull_up_input();
    // let b_pin = pins.gp4.into_pull_up_input();
    let b = b_pin.as_input();

    let mut hw_inputs = McInputs::new(a, b, up, down, left, right);

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

    let mut inputs = FrugInputs::default();
    display.set_rotation(DisplayRotation::Rotate90);

    // let mut menu=  Menu::new(|| unsafe { DATA_STORAGE }, |offset, data| unsafe { DATA_STORAGE }.copy_from_slice(&data));
    let mut menu = Menu::new(|| unsafe { DATA_STORAGE }, |offset, data| {});

    let mut logic_avg = RollingAverage::new();
    let target_fps = 60;

    loop {
        let frame_start = timer.get_counter().ticks();

        // TODO detect x frames held for restart/pause

        // Update inputs
        hw_inputs.tick(&mut inputs);

        menu.update(&mut inputs);

        let logic_end = timer.get_counter().ticks();
        let logic_time = logic_end - frame_start;
        logic_avg.add(logic_time);

        menu.frugger().draw_frame(&mut display);
        let _ = display.flush();

        let draw_end = timer.get_counter().ticks();
        let draw_time = draw_end - logic_end;
        let total_time = draw_end - frame_start;

        // TODO render fps?
        if total_time < target_fps {
            timer.delay_ms((target_fps - total_time) as u32);
        }
    }
}
