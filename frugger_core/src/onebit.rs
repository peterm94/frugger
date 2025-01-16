use core::convert::Infallible;
use core::mem;

use embedded_graphics::geometry::Dimensions;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::Pixel;

use crate::{FruggerEngine, Orientation};

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
