use crate::mc_inputs::McInputs;
use fugit::RateExtU32;
use sh1106::prelude::*;
use sh1106::Builder;

use crate::RollingAverage;
use bsp::hal::clocks::SystemClock;
use bsp::hal::{Sio, Timer};
use bsp::pac;
use cortex_m::prelude::_embedded_hal_blocking_delay_DelayMs;
use embedded_graphics::mono_font::ascii::FONT_5X8;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle, StyledDrawable};
use embedded_graphics::text::Text;
use frugger_core::{ButtonInput, FrugInputs, FruggerEngine, FruggerGame};
use input_test_small::InputTestSmall;
use numtoa::NumToA;
use runner::Runner;
use triangle_jump::Jump;
use waveshare_rp2040_zero as bsp;

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

    let mut hw_inputs = McInputs::new(a, b, up, down, left, right);
    let mut frug_inputs = FrugInputs::default();

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

    type GAME = Jump;
    let mut game = GAME::new(timer.get_counter().ticks());

    let target_fps = 1000 / GAME::TARGET_FPS;

    let mut logic_avg = RollingAverage::new();

    let mut buf = [0u8; 20];
    let fps_backing = Rectangle::new(Point::new(0, 0), Size::new(10, 9));
    let rect_style = PrimitiveStyle::with_fill(BinaryColor::Off);
    let txt_style = MonoTextStyle::new(&FONT_5X8, BinaryColor::On);

    // Start game loop
    loop {
        let frame_start = timer.get_counter();

        hw_inputs.tick(&mut frug_inputs);

        game.update(&frug_inputs);

        let logic_end = timer.get_counter();
        let logic_time = (logic_end - frame_start).to_millis();

        logic_avg.add(logic_time);

        game.frugger().draw_frame(&mut display);
        display.flush().unwrap();

        let draw_end = timer.get_counter();
        let draw_time = (draw_end - logic_end).to_millis();
        let total_time = (draw_end - frame_start).to_millis();
        log!("{}", draw_time);
        // log!("Logic: {logic_time} Draw: {draw_time}, Total: {total_time} / {target_fps}");

        let frame_time = total_time.numtoa_str(10, &mut buf);
        fps_backing.draw_styled(&rect_style, &mut display).unwrap();
        let text = Text::new(frame_time, Point::new(0, 15), txt_style);
        text.draw(&mut display).unwrap();

        if total_time < target_fps {
            timer.delay_ms((target_fps - logic_time) as u32);
        }
    }
}
