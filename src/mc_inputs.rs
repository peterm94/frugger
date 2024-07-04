use embedded_hal::digital::v2::InputPin;
use frugger_core::{ButtonInput, ButtonState, FrugInputs};

pub struct McInputs<A: InputPin, B: InputPin, L: InputPin, R: InputPin, U: InputPin, D: InputPin> {
    a_pin: A,
    b_pin: B,
    left_pin: L,
    right_pin: R,
    up_pin: U,
    down_pin: D,
}
impl<A: InputPin, B: InputPin, L: InputPin, R: InputPin, U: InputPin, D: InputPin> McInputs<A, B, L, R, U, D> {
    pub fn new(a_pin: A, b_pin: B, left_pin: L, right_pin: R, up_pin: U, down_pin: D) -> Self {
        Self {
            a_pin,
            b_pin,
            left_pin,
            right_pin,
            up_pin,
            down_pin,
        }
    }

    fn set_button_state<P: InputPin>(pin: &P, button: &mut ButtonState) {

        // Active
        if pin.is_low().ok().unwrap() {
            match button {
                ButtonState::RELEASED | ButtonState::UP => *button = ButtonState::PRESSED,
                _ => {}
            }
            // Inactive
        } else if pin.is_high().ok().unwrap() {
            match button {
                ButtonState::DOWN | ButtonState::PRESSED => *button = ButtonState::RELEASED,
                _ => {}
            }
        }
    }
}

impl<A: InputPin, B: InputPin, L: InputPin, R: InputPin, U: InputPin, D: InputPin> ButtonInput for McInputs<A, B, L, R, U, D> {
    fn tick(&mut self, inputs: &mut FrugInputs) {
        // set based on last frame
        for button in [&mut inputs.a, &mut inputs.b, &mut inputs.up, &mut inputs.down, &mut inputs.left, &mut inputs.right] {
            match button {
                ButtonState::PRESSED => { *button = ButtonState::DOWN }
                ButtonState::RELEASED => { *button = ButtonState::UP }
                ButtonState::DOWN => {}
                ButtonState::UP => {}
            }
        }
        // update based on this frame
        Self::set_button_state(&self.a_pin, &mut inputs.a);
        Self::set_button_state(&self.b_pin, &mut inputs.b);
    }
}