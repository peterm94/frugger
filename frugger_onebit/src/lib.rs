#![cfg_attr(not(test), no_std)]

mod games;

use core::convert::Infallible;
use core::mem;

use crate::games::triangle_jump::Jump;
use embedded_graphics::geometry::Dimensions;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::Pixel;
use frugger_core::util::RollingAverage;
use frugger_core::{FrugDisplay, FrugInputs, FrugTimer, FruggerEngine, FruggerGame, Orientation};

// pub trait HasThings {
//     fn set_orientation(&mut self, orientation: &Orientation);
//     fn flush(&mut self);
// }
pub struct OneBitDisplay<T>(pub T)
where
    T: DrawTarget<Color = BinaryColor> + FrugDisplay;

pub struct OneBitRunner<'a, G, T>
where
    G: Fn(&mut FrugInputs),
    T: DrawTarget<Color = BinaryColor> + FrugDisplay,
{
    save_ptr: *mut u8,
    display: OneBitDisplay<T>,
    update_input: G,
    timer: &'a mut dyn FrugTimer,
    target_fps: u64,
}

impl<'a, G, T> OneBitRunner<'a, G, T>
where
    G: Fn(&mut FrugInputs),
    T: DrawTarget<Color = BinaryColor> + FrugDisplay,
{
    pub fn new(
        save_ptr: *mut u8,
        display: OneBitDisplay<T>,
        update_input: G,
        timer: &'a mut dyn FrugTimer,
    ) -> Self {
        Self {
            save_ptr,
            display,
            update_input,
            timer,
            target_fps: 60,
        }
    }

    pub fn start(&mut self) -> ! {
        let mut inputs = FrugInputs::default();
        // Default portrait
        self.display.0.set_orientation(&Orientation::Portrait);

        let mut game = Jump::new(self.timer.ticks());

        let mut logic_avg = RollingAverage::new();

        loop {
            let frame_start = self.timer.ticks();

            // Update inputs
            (self.update_input)(&mut inputs);

            game.update(&mut inputs);

            let logic_end = self.timer.ticks();
            let logic_time = logic_end - frame_start;
            logic_avg.add(logic_time);

            // TODO draw game
            game.frugger().draw_frame(&mut self.display.0);

            self.display.0.flush();

            let draw_end = self.timer.ticks();
            let draw_time = draw_end - logic_end;
            let total_time = draw_end - frame_start;

            // TODO render fps?
            if total_time < self.target_fps {
                self.timer.delay_ms(self.target_fps - total_time);
            }
        }
    }
}

pub struct OneBit {
    last_frame: [BinaryColor; 8192],
    next_frame: [BinaryColor; 8192],
    scr_width: usize,
    orientation: Orientation,
}

impl OneBit {
    pub fn new(orientation: Orientation) -> Self {
        Self {
            last_frame: [BinaryColor::Off; 8192],
            next_frame: [BinaryColor::Off; 8192],
            scr_width: match orientation {
                Orientation::Landscape => 128,
                Orientation::Portrait => 64,
            },
            orientation,
        }
    }
}

impl Dimensions for OneBit {
    fn bounding_box(&self) -> Rectangle {
        match self.orientation {
            Orientation::Landscape => Rectangle::new(Point::new(0, 0), Size::new(128, 64)),
            Orientation::Portrait => Rectangle::new(Point::new(0, 0), Size::new(64, 128)),
        }
    }
}

impl DrawTarget for OneBit {
    type Color = BinaryColor;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let Size { width, height } = self.bounding_box().size;
        for Pixel(point, col) in pixels {
            if point.x < 0
                || point.x > (width - 1) as i32
                || point.y < 0
                || point.y > (height - 1) as i32
            {
                continue;
            }
            let idx = (point.y * width as i32 + point.x) as usize;
            self.next_frame[idx] = col;
        }
        Ok(())
    }
}

impl FruggerEngine<BinaryColor> for OneBit {
    fn draw_frame<T>(&mut self, display: &mut T)
    where
        T: DrawTarget<Color = BinaryColor>,
    {
        for (idx, col) in self.next_frame.iter().enumerate() {
            let x = idx % self.scr_width;
            let y = idx / self.scr_width;
            if &self.last_frame[idx] != col {
                display.fill_solid(
                    &Rectangle::new(Point::new(x as _, y as _), Size::new_equal(1)),
                    *col,
                );
            }
        }

        mem::swap(&mut self.next_frame, &mut self.last_frame);
        self.next_frame.fill(BinaryColor::Off);
    }
}
