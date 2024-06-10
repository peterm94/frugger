#![cfg_attr(not(test), no_std)]

use core::convert::Infallible;

use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Dimensions, Point, Size};
use embedded_graphics::mono_font::ascii::{FONT_6X10, FONT_8X13};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::Pixel;
use embedded_graphics::pixelcolor::{PixelColor, Rgb565};
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::text::Text;
use heapless::String;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum FruggerColour {
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

pub struct Frugger<'a, T> where T: DrawTarget<Color=Rgb565> {
    // 320 * 240 / 2
    default_val: u8,
    last_frame: [u8; 38400],
    next_frame: [u8; 38400],
    display: &'a mut T,
}

impl<T> Dimensions for Frugger<'_, T> where T: DrawTarget<Color=Rgb565> {
    fn bounding_box(&self) -> Rectangle {
        Rectangle::new(Point::new(0, 0), Size::new(320, 240))
    }
}

impl<T> DrawTarget for Frugger<'_, T> where T: DrawTarget<Color=Rgb565> {
    type Color = FruggerColour;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error> where I: IntoIterator<Item=Pixel<Self::Color>> {
        for Pixel(point, col) in pixels {
            self.write_pixel_value(point.x as u16, point.y as u16, col)
            // let rect = Rectangle::new(Point::new(point.x as i32, point.y as i32), Size::new(1, 1));
            // self.display.fill_solid(&rect, col.rgb565());
        }

        Ok(())
    }
}

impl<'a, T> Frugger<'a, T> where
    T: DrawTarget<Color=Rgb565> {
    pub fn new(bg_col: FruggerColour, display: &'a mut T) -> Self {
        let default_val = bg_col.bits() | (bg_col.bits() << 4);
        Self {
            default_val,
            last_frame: [u8::MAX; 38400],
            next_frame: [default_val; 38400],
            display,
        }
    }
    fn get_pixel_value(&self, x: u16, y: u16) -> FruggerColour {
        let pixel_offset = (y as u32 * 320 + x as u32) as usize;

        // If it's even, we can half it and read the first 4 bits of the byte at the index
        let colour = if pixel_offset % 2 == 0 {
            self.last_frame[pixel_offset / 2] & 0x0F
        } else {
            // Odd number, we have to read bits 5-8
            self.last_frame[pixel_offset / 2] >> 4
        };

        FruggerColour::idx(colour)
    }

    fn get_pixel_value_next(&self, x: u16, y: u16) -> FruggerColour {
        let pixel_offset = (y as u32 * 320 + x as u32) as usize;

        // If it's even, we can half it and read the first 4 bits of the byte at the index
        let colour = if pixel_offset % 2 == 0 {
            self.next_frame[pixel_offset / 2] & 0x0F
        } else {
            // Odd number, we have to read bits 5-8
            self.next_frame[pixel_offset / 2] >> 4
        };

        FruggerColour::idx(colour)
    }

    fn write_pixel_value(&mut self, x: u16, y: u16, colour: FruggerColour) {
        if x >= 320 || y >= 240 { return; }

        let pixel_offset = (y as u32 * 320 + x as u32) as usize;
        let value = colour.bits() & 0x0F;

        if pixel_offset % 2 == 0 {
            self.next_frame[pixel_offset / 2] = (self.next_frame[pixel_offset / 2] & 0xF0) | value;
        } else {
            self.next_frame[pixel_offset / 2] = (self.next_frame[pixel_offset / 2] & 0x0F) | (value << 4);
        }
    }

    pub fn draw_frame(&mut self)
    {
        let mut changed = 0;
        for y in 0..240 {
            for x in 0..320 {
                let next = self.get_pixel_value_next(x, y);
                let last = self.get_pixel_value(x, y);
                if next != last {
                    let rect = Rectangle::new(Point::new(x as i32, y as i32), Size::new(1, 1));
                    self.display.fill_solid(&rect, next.rgb565());
                    changed += 1;
                }
            }
        }

        let text_style = MonoTextStyle::new(&FONT_8X13, FruggerColour::White);
        let ch = String::<10>::try_from(changed).unwrap();

        // Override last frame with the new one
        self.last_frame.copy_from_slice(&self.next_frame);
        self.next_frame.fill(self.default_val);
        Text::new(ch.as_str(), Point::new(100, 100), text_style).draw(self);
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
            .stroke_color(FruggerColour::Red)
            .stroke_width(3)
            .fill_color(FruggerColour::Green)
            .build();
        let f_style2 = PrimitiveStyleBuilder::new()
            .stroke_color(FruggerColour::Purple)
            .stroke_width(3)
            .fill_color(FruggerColour::Orange)
            .build();

        let mut frugger = Frugger::new(FruggerColour::Blue, &mut display);

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

        frugger.draw_frame();

        Window::new("Hello World", &output_settings).show_static(&display);
    }
}