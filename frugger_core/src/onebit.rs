use core::convert::Infallible;
use core::mem;

use embedded_graphics::geometry::Dimensions;
use embedded_graphics::Pixel;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;

use crate::FruggerEngine;

pub struct OneBit {
    frame_data: [BinaryColor; 8192],
}

impl OneBit {
    pub fn new() -> Self {
        Self {
            frame_data: [BinaryColor::Off; 8192]
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
            let idx = (point.y * 128 + point.x) as usize;
            if idx > 8192 { continue; }
            self.frame_data[idx] = col;
        }

        Ok(())
    }
}

impl FruggerEngine<BinaryColor> for OneBit {
    fn draw_frame<T>(&mut self, display: &mut T) where T: DrawTarget<Color=BinaryColor> {
        let frame = mem::replace(&mut self.frame_data, [BinaryColor::Off; 8192]);
        display.fill_contiguous(&self.bounding_box(), frame);
    }
}