use crate::OneBit;
use embedded_graphics::image::Image;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;
use frugger_core::FrugInputs;
use heapless::LinearMap;
use tinybmp::Bmp;

type UpdateFn<C> = fn(&mut C, &FrugInputs, &mut OneBit) -> usize;

/// State machine.
pub struct SM<C> {
    curr: usize,
    states: LinearMap<usize, UpdateFn<C>, 100>,
}

impl<C> SM<C> {
    pub fn new() -> Self {
        Self {
            states: LinearMap::new(),
            curr: 0,
        }
    }

    pub fn add(&mut self, id: usize, state: UpdateFn<C>) {
        self.states.insert(id, state);
    }

    pub fn tick(&mut self, state: &mut C, inputs: &FrugInputs, engine: &mut OneBit) {
        self.curr = if let Some(func) = self.states.get(&self.curr) {
            func(state, inputs, engine)
        } else {
            self.curr
        }
    }
}

pub struct Spr<'a> {
    bmp: Bmp<'a, BinaryColor>,
}

impl<'a> Spr<'a> {
    pub fn new(bmp: Bmp<'a, BinaryColor>) -> Self {
        Spr { bmp }
    }

    fn draw_at(&self, pos: &Point, target: &mut OneBit) {
        Image::new(&self.bmp, *pos).draw(target).unwrap();
    }
}
pub fn load_sprite<'a>(bytes: &'a [u8]) -> Spr<'a> {
    let bmp = Bmp::from_slice(bytes).unwrap();
    Spr::new(bmp)
}

pub struct Sprite<'a> {
    area: Rectangle,
    data: &'a [u8],
}

impl<'a> Sprite<'a> {
    pub const fn new(
        width: u32,
        height: u32,
        anchor_x: i32,
        anchor_y: i32,
        data: &'a [u8],
    ) -> Self {
        let area = Rectangle::new(Point::new(anchor_x, anchor_y), Size::new(width, height));
        Self { area, data }
    }
}

impl Drawable for Sprite<'_> {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        target.fill_contiguous(
            &self.area,
            self.data.iter().map(|x| {
                if *x == 0 {
                    BinaryColor::Off
                } else {
                    BinaryColor::On
                }
            }),
        )
    }
}
