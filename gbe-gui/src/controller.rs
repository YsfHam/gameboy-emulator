use std::collections::{HashMap, HashSet};

use eframe::egui;
use gbe_core::joypad::JoypButton;
use gilrs::{Button, Gilrs};


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
    pub fn poll(&self, ctx: &egui::Context) -> Option<(JoypButton, KeyState)> {
        ctx.input(|input| {
            for (key, joyp) in &self.bindings {
                if input.key_pressed(*key) {
                    return Some((*joyp, KeyState::Pressed))
                }
                
                if input.key_released(*key) {
                    return Some((*joyp, KeyState::Released))
                }
            }
            None
        })
    }
}


struct GamepadController {
    gilrs: Gilrs,
    pressed_buttons: HashSet<Button>
}

impl GamepadController {
    fn new() -> Option<Self> {
        Some(Self {
            gilrs: Gilrs::new().ok()?,
            pressed_buttons: HashSet::new(),
        })
    }

    fn pull_events(&mut self) {
        while self.gilrs.next_event().is_some() {}

    }

    fn is_pressed(&mut self, button: Button) -> bool {
        let pressed = 
            self.gilrs.gamepads().next()
            .map(|(_, pad)| pad.is_pressed(button)).unwrap_or(false);

        if pressed {
            self.pressed_buttons.insert(button);
            true
        }
        else {
            false
        }
    }

    fn is_released(&mut self, button: Button) -> bool {
        if self.is_pressed(button) {
            false
        }
        else {
            self.pressed_buttons.remove(&button)
        }
    }
}

pub struct GamepadBindings {
    bindings: HashMap<Button, JoypButton>,
}

impl Default for GamepadBindings {
    fn default() -> Self {
        let bindings_list = [
            (Button::South, JoypButton::A),
            (Button::East, JoypButton::B),
            (Button::Start, JoypButton::Start),
            (Button::Select, JoypButton::Select),
            (Button::DPadUp, JoypButton::Up),
            (Button::DPadDown, JoypButton::Down),
            (Button::DPadLeft, JoypButton::Left),
            (Button::DPadRight, JoypButton::Right),
        ];

        Self {
            bindings: bindings_list.into()
        }
    }
}

impl GamepadBindings {
    fn poll(&self, controller: &mut GamepadController) -> Option<(JoypButton, KeyState)> {
        controller.pull_events();
        for (key, joyp) in &self.bindings {
            if controller.is_pressed(*key) {
                return Some((*joyp, KeyState::Pressed))
            }
            
            if controller.is_released(*key) {
                return Some((*joyp, KeyState::Released))
            }
        }
        None
    }
}

pub struct Controllers {
    keyboard: KeyboardBindings,
    gamepad_bindings: GamepadBindings,
    gamepad_controller: Option<GamepadController>,
}

impl Controllers {
    pub fn new() -> Self  {
        Self {
            keyboard: KeyboardBindings::default(),
            gamepad_bindings: GamepadBindings::default(),
            gamepad_controller: GamepadController::new()
        }
    }

    pub fn poll(&mut self, ctx: &egui::Context) -> Option<(JoypButton, KeyState)> {
        let gamepad_controller = self.gamepad_controller.as_mut();
        let gamepad_bindings = &self.gamepad_bindings;
        self
            .keyboard.poll(ctx)
            .or_else(|| {
                gamepad_controller
                .and_then(|controller| 
                    gamepad_bindings.poll(controller)
                )
            })
    }
}