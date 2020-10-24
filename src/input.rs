use std::collections::HashMap;
use ultraviolet::Vec2;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};

#[derive(Default)]
pub struct Mouse {
    pub acceleration: Vec2,
    pub position: Vec2,
    pub old_position: Vec2,
}

#[derive(Default)]
pub struct KeyboardKey {
    pressed: bool,
    held: bool,
    released: bool,
}

#[derive(Default)]
pub struct Keyboard {
    pub keys: HashMap<VirtualKeyCode, KeyboardKey>,
}

#[derive(Default)]
pub struct Input {
    pub keyboard: Keyboard,
    pub mouse: Mouse,
}

impl Input {
    pub fn process_keyboard(&mut self, keyboard_input: &KeyboardInput) {
        let virtual_keycode = keyboard_input
            .virtual_keycode
            .expect("keyboard input doesn't have virtual keycode");

        if !self.keyboard.keys.contains_key(&virtual_keycode) {
            self.keyboard
                .keys
                .insert(virtual_keycode, KeyboardKey::default());
        }

        let keyboard_key = self
            .keyboard
            .keys
            .get_mut(&virtual_keycode)
            .expect("failed retrieving key from keyboard keys map!");

        keyboard_key.held = if keyboard_input.state == ElementState::Pressed {
            true
        } else {
            false
        };
    }

    pub fn key_pressed(&self, key_code: VirtualKeyCode) -> bool {
        false
    }

    pub fn key_held(&self, key_code: VirtualKeyCode) -> bool {
        self.keyboard
            .keys
            .get(&key_code)
            .map_or(false, |key| key.held)
    }

    pub fn key_released(&self, key_code: VirtualKeyCode) -> bool {
        false
    }
}
