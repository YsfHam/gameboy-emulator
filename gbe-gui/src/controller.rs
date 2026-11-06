use std::collections::HashMap;

use eframe::egui;
use gbe_core::joypad::JoypButton;


pub enum KeyState {
    Pressed,
    Released,
}

pub struct KeyboardBindings {
    bindings: HashMap<egui::Key, JoypButton>,
}

impl Default for KeyboardBindings {
    fn default() -> Self {
        let bindings_list = [
            (egui::Key::A, JoypButton::A),
            (egui::Key::Z, JoypButton::B),
            (egui::Key::S, JoypButton::Start),
            (egui::Key::Q, JoypButton::Select),
            (egui::Key::ArrowUp, JoypButton::Up),
            (egui::Key::ArrowDown, JoypButton::Down),
            (egui::Key::ArrowLeft, JoypButton::Left),
            (egui::Key::ArrowRight, JoypButton::Right),
        ];

        Self {
            bindings: bindings_list.into()
        }
    }
}

impl KeyboardBindings {
    pub fn poll(&self, input: &egui::InputState) -> Option<(JoypButton, KeyState)> {
        for (key, joyp) in &self.bindings {
            if input.key_pressed(*key) {
                return Some((*joyp, KeyState::Pressed))
            }

            if input.key_released(*key) {
                return Some((*joyp, KeyState::Released))
            }
        }
        None
    }
}