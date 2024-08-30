use core::convert::Infallible;
use core::mem;

use embedded_graphics::geometry::Dimensions;
use embedded_graphics::Pixel;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;

use crate::FruggerEngine;

pub struct OneBit {
    last_frame: [BinaryColor; 8192],
    next_frame: [BinaryColor; 8192],
}

impl OneBit {
    pub fn new() -> Self {
        Self {
            last_frame: [BinaryColor::Off; 8192],
            next_frame: [BinaryColor::Off; 8192],
        }
    }
}


impl Dimensions for OneBit {
    fn bounding_box(&self) -> Rectangle {
        Rectangle::new(Point::new(0, 0), Size::new(128, 64))
    }
}

impl DrawTarget for OneBit {
    type Color = BinaryColor;
    type Error = Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error> where I: IntoIterator<Item=Pixel<Self::Color>> {
        for Pixel(point, col) in pixels {
            if point.x < 0 || point.x > 127 || point.y < 0 || point.y > 63 { continue; }
            let idx = (point.y * 128 + point.x) as usize;
            self.next_frame[idx] = col;
        }
        Ok(())
    }
}

impl FruggerEngine<BinaryColor> for OneBit {
    fn draw_frame<T>(&mut self, display: &mut T) where T: DrawTarget<Color=BinaryColor> {
        for (idx, col) in self.next_frame.iter().enumerate() {
            let x = idx % 128;
            let y = idx / 128;
            if &self.last_frame[idx] != col {
                display.fill_solid(&Rectangle::new(Point::new(x as _, y as _), Size::new_equal(1)), *col);
            }
        }

        mem::swap(&mut self.next_frame, &mut self.last_frame);
        self.next_frame.fill(BinaryColor::Off);
    }
}