#![cfg_attr(not(test), no_std)]

use core::convert::Infallible;

use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Dimensions, Point, Size};
use embedded_graphics::Pixel;
use embedded_graphics::pixelcolor::{PixelColor, Rgb565};
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;

mod game;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Palette {
    Black,
    Purple,
    Red,
    Orange,
    Yellow,
    Lime,
    Green,
    Teal,
    NavyBlue,
    DarkBlue,
    Blue,
    LightBlue,
    White,
    LightGrey,
    DarkGrey,
    BlueGrey,
}

impl Into<Rgb565> for Palette {
    fn into(self) -> Rgb565 {
        match self {
            Palette::Black => Rgb565::new(3, 7, 5),
            Palette::Purple => Rgb565::new(11, 10, 11),
            Palette::Red => Rgb565::new(22, 15, 10),
            Palette::Orange => Rgb565::new(29, 31, 11),
            Palette::Yellow => Rgb565::new(31, 51, 14),
            Palette::Lime => Rgb565::new(20, 59, 14),
            Palette::Green => Rgb565::new(7, 45, 12),
            Palette::Teal => Rgb565::new(4, 28, 15),
            Palette::NavyBlue => Rgb565::new(5, 13, 13),
            Palette::DarkBlue => Rgb565::new(7, 23, 24),
            Palette::Blue => Rgb565::new(8, 41, 30),
            Palette::LightBlue => Rgb565::new(14, 59, 30),
            Palette::White => Rgb565::new(30, 60, 30),
            Palette::LightGrey => Rgb565::new(18, 43, 24),
            Palette::DarkGrey => Rgb565::new(10, 27, 16),
            Palette::BlueGrey => Rgb565::new(6, 15, 11)
        }
    }
}

impl Palette {
    fn from_index(idx: u8) -> Self {
        match idx {
            0 => Palette::Black,
            1 => Palette::Purple,
            2 => Palette::Red,
            3 => Palette::Orange,
            4 => Palette::Yellow,
            5 => Palette::Lime,
            6 => Palette::Green,
            7 => Palette::Teal,
            8 => Palette::NavyBlue,
            9 => Palette::DarkBlue,
            10 => Palette::Blue,
            11 => Palette::LightBlue,
            12 => Palette::White,
            13 => Palette::LightGrey,
            14 => Palette::DarkGrey,
            15 => Palette::BlueGrey,
            _ => panic!()
        }
    }
    fn bits(&self) -> u8 {
        *self as u8
    }
}

impl PixelColor for Palette {
    type Raw = ();
}

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

        Palette::from_index(colour)
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

        Palette::from_index(colour)
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

    pub fn draw_frame<T>(&mut self, display: &mut T) where T: DrawTarget<Color=Rgb565> {
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


#[cfg(test)]
mod tests {
    use embedded_graphics::primitives::PrimitiveStyleBuilder;
    use embedded_graphics_simulator::{BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, Window};

    use super::*;

    #[test]
    fn it_works() {
        let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(320, 240));

        let output_settings = OutputSettingsBuilder::new()
            .theme(BinaryColorTheme::Default)
            .build();

        let f_style = PrimitiveStyleBuilder::new()
            .stroke_color(Palette::Red)
            .stroke_width(3)
            .fill_color(Palette::Green)
            .build();
        let f_style2 = PrimitiveStyleBuilder::new()
            .stroke_color(Palette::Purple)
            .stroke_width(3)
            .fill_color(Palette::Orange)
            .build();

        let mut frugger = Frugger::new(Palette::Blue);

        let rec2 = Rectangle::new(Point::new(0, 0), Size::new(10, 10))
            .into_styled(f_style);
        rec2.draw(&mut frugger);
        let rec2 = Rectangle::new(Point::new(0, 230), Size::new(10, 10))
            .into_styled(f_style);
        rec2.draw(&mut frugger);
        let rec2 = Rectangle::new(Point::new(310, 0), Size::new(10, 10))
            .into_styled(f_style2);
        rec2.draw(&mut frugger);
        let rec2 = Rectangle::new(Point::new(310, 230), Size::new(10, 10))
            .into_styled(f_style);
        rec2.draw(&mut frugger);

        frugger.draw_frame(&mut display);

        Window::new("Hello World", &output_settings).show_static(&display);
    }
}