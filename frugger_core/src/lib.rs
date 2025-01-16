#![cfg_attr(not(test), no_std)]

use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::Dimensions;
use embedded_graphics::pixelcolor::{PixelColor, Rgb565};
use embedded_graphics::prelude::*;

pub mod frugger;
pub mod onebit;

pub trait FruggerEngine<C> {
    fn draw_frame<T>(&mut self, display: &mut T) where T: DrawTarget<Color=C>;
}

#[derive(Default, Eq, PartialEq)]
pub enum ButtonState {
    PRESSED,
    RELEASED,
    DOWN,
    #[default]
    UP,
}

impl ButtonState {
    /// Already down or pressed this frame.
    pub fn down(&self) -> bool {
        self == &ButtonState::PRESSED || self == &ButtonState::DOWN
    }

    /// Already up or released this frame.
    pub fn up(&self) -> bool {
        self == &ButtonState::UP || self == &ButtonState::RELEASED
    }

    /// Pressed this frame.
    pub fn pressed(&self) -> bool {
        self == &ButtonState::PRESSED
    }

    /// Release this frame.
    pub fn released(&self) -> bool {
        self == &ButtonState::RELEASED
    }
}

pub trait ButtonInput {
    fn tick(&mut self, inputs: &mut FrugInputs);
}

#[derive(Default, Eq, PartialEq)]
pub struct FrugInputs {
    pub a: ButtonState,
    pub b: ButtonState,
    pub left: ButtonState,
    pub right: ButtonState,
    pub up: ButtonState,
    pub down: ButtonState,
}

pub enum Orientation {
    Landscape,
    Portrait
}

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

impl Palette {
    const BlackC: Rgb565 = Rgb565::new(3, 7, 5);
    const PurpleC: Rgb565 = Rgb565::new(11, 10, 11);
    const RedC: Rgb565 = Rgb565::new(22, 15, 10);
    const OrangeC: Rgb565 = Rgb565::new(29, 31, 11);
    const YellowC: Rgb565 = Rgb565::new(31, 51, 14);
    const LimeC: Rgb565 = Rgb565::new(20, 59, 14);
    const GreenC: Rgb565 = Rgb565::new(7, 45, 12);
    const TealC: Rgb565 = Rgb565::new(4, 28, 15);
    const NavyBlueC: Rgb565 = Rgb565::new(5, 13, 13);
    const DarkBlueC: Rgb565 = Rgb565::new(7, 23, 24);
    const BlueC: Rgb565 = Rgb565::new(8, 41, 30);
    const LightBlueC: Rgb565 = Rgb565::new(14, 59, 30);
    const WhiteC: Rgb565 = Rgb565::new(30, 60, 30);
    const LightGreyC: Rgb565 = Rgb565::new(18, 43, 24);
    const DarkGreyC: Rgb565 = Rgb565::new(10, 27, 16);
    const BlueGreyC: Rgb565 = Rgb565::new(6, 15, 11);
}

impl Into<Rgb565> for Palette {
    fn into(self) -> Rgb565 {
        match self {
            Palette::Black => Self::BlackC,
            Palette::Purple => Self::PurpleC,
            Palette::Red => Self::RedC,
            Palette::Orange => Self::OrangeC,
            Palette::Yellow => Self::YellowC,
            Palette::Lime => Self::LimeC,
            Palette::Green => Self::GreenC,
            Palette::Teal => Self::TealC,
            Palette::NavyBlue => Self::NavyBlueC,
            Palette::DarkBlue => Self::DarkBlueC,
            Palette::Blue => Self::BlueC,
            Palette::LightBlue => Self::LightBlueC,
            Palette::White => Self::WhiteC,
            Palette::LightGrey => Self::LightGreyC,
            Palette::DarkGrey => Self::DarkGreyC,
            Palette::BlueGrey => Self::BlueGreyC,
        }
    }
}

impl Palette {
    pub fn from_index(idx: &u8) -> Option<Self> {
        match idx {
            0 => Some(Palette::Black),
            1 => Some(Palette::Purple),
            2 => Some(Palette::Red),
            3 => Some(Palette::Orange),
            4 => Some(Palette::Yellow),
            5 => Some(Palette::Lime),
            6 => Some(Palette::Green),
            7 => Some(Palette::Teal),
            8 => Some(Palette::NavyBlue),
            9 => Some(Palette::DarkBlue),
            10 => Some(Palette::Blue),
            11 => Some(Palette::LightBlue),
            12 => Some(Palette::White),
            13 => Some(Palette::LightGrey),
            14 => Some(Palette::DarkGrey),
            15 => Some(Palette::BlueGrey),
            _ => None
        }
    }

    fn bits(&self) -> u8 {
        *self as u8
    }
}

impl PixelColor for Palette {
    type Raw = ();
}

pub trait FruggerGame {
    const TARGET_FPS: u64;

    const ORIENTATION: Orientation;

    type Color: PixelColor;
    type Engine: FruggerEngine<Self::Color>;
    fn update(&mut self, inputs: &FrugInputs);
    fn frugger(&mut self) -> &mut Self::Engine;
}
