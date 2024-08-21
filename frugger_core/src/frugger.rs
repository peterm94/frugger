use core::convert::Infallible;

use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Dimensions, Point, Size};
use embedded_graphics::Pixel;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::RgbColor;
use embedded_graphics::primitives::Rectangle;

use crate::{FruggerEngine, Palette};

pub struct Frugger {
    // 320 * 240 / 2
    default_val: u8,
    last_frame: [u8; 38400],
    next_frame: [u8; 38400],
}

impl Dimensions for Frugger {
    fn bounding_box(&self) -> Rectangle {
        Rectangle::new(Point::new(0, 0), Size::new(320, 240))
    }
}

impl DrawTarget for Frugger {
    type Color = Palette;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error> where I: IntoIterator<Item=Pixel<Self::Color>> {
        for Pixel(point, col) in pixels {
            self.write_pixel_value(point.x as u16, point.y as u16, col)
        }
        Ok(())
    }
}

impl FruggerEngine<Rgb565> for Frugger {
    fn draw_frame<T>(&mut self, display: &mut T) where T: DrawTarget<Color=Rgb565> {
        let mut cols = [Rgb565::BLACK; 320];

        // iterate over rows and draw continuous segments
        // todo we can cut the screen into a grid?
        // todo make sure the draw direction is the one we actually want for the display
        for y in 0..240 {
            let mut run_start: i32 = -1;
            let mut run_length = 0;
            for x in 0..320 {
                let next = self.get_pixel_value_next(x, y);
                let last = self.get_pixel_value(x, y);

                if next != last {
                    if run_start == -1 { run_start = x as _; }
                    cols[run_length] = next.into();
                    run_length += 1;
                } else if run_start != -1 && (next == last) {
                    let area = Rectangle::new(Point::new(run_start, y as _), Size::new(run_length as _, 1));
                    display.fill_contiguous(&area, cols);

                    run_length = 0;
                    run_start = -1;
                }

                if x == 319 && run_start != -1 {
                    let area = Rectangle::new(Point::new(run_start, y as _), Size::new(run_length as _, 1));
                    display.fill_contiguous(&area, cols);
                }
            }
        }

        self.last_frame.copy_from_slice(&self.next_frame);
        self.next_frame.fill(self.default_val);
    }
}

impl Frugger {
    pub fn new(bg_col: Palette) -> Self {
        let default_val = bg_col.bits() | (bg_col.bits() << 4);
        Self {
            default_val,
            last_frame: [u8::MAX; 38400],
            next_frame: [default_val; 38400],
        }
    }
    fn get_pixel_value(&self, x: u16, y: u16) -> Palette {
        let pixel_offset = (y as u32 * 320 + x as u32) as usize;

        // If it's even, we can half it and read the first 4 bits of the byte at the index
        let colour = if pixel_offset % 2 == 0 {
            self.last_frame[pixel_offset / 2] & 0x0F
        } else {
            // Odd number, we have to read bits 5-8
            self.last_frame[pixel_offset / 2] >> 4
        };

        Palette::from_index(&colour).unwrap()
    }

    fn get_pixel_value_next(&self, x: u16, y: u16) -> Palette {
        let pixel_offset = (y as u32 * 320 + x as u32) as usize;

        // If it's even, we can half it and read the first 4 bits of the byte at the index
        let colour = if pixel_offset % 2 == 0 {
            self.next_frame[pixel_offset / 2] & 0x0F
        } else {
            // Odd number, we have to read bits 5-8
            self.next_frame[pixel_offset / 2] >> 4
        };

        Palette::from_index(&colour).unwrap()
    }

    fn write_pixel_value(&mut self, x: u16, y: u16, colour: Palette) {
        if x >= 320 || y >= 240 { return; }

        let pixel_offset = (y as u32 * 320 + x as u32) as usize;
        let value = colour.bits() & 0x0F;

        if pixel_offset % 2 == 0 {
            self.next_frame[pixel_offset / 2] = (self.next_frame[pixel_offset / 2] & 0xF0) | value;
        } else {
            self.next_frame[pixel_offset / 2] = (self.next_frame[pixel_offset / 2] & 0x0F) | (value << 4);
        }
    }
}