pub mod button;
pub mod cursor;
use crate::utils::unwrap_mut;
use std::collections::HashMap;

/// Returns true iff the button is down (i.e was pressed this frame or is being held).
pub fn is_button_down(ctx: &mut crate::EngineContext, button_id: &button::ButtonId) -> bool {
    let state = unwrap_mut(&mut ctx.input_system).button_state_or_default(button_id);

    match state {
        button::ButtonState::Up { .. } => false,
        button::ButtonState::Down { .. } => true,
    }
}

/// Returns true iff the button was pressed THIS SPECIFIC FRAME, but false for all other frames if the user
/// continues to hold it down.
pub fn is_button_pressed(ctx: &mut crate::EngineContext, button_id: &button::ButtonId) -> bool {
    let curr_frame = unwrap_mut(&mut ctx.input_system).frame_count;
    let state = unwrap_mut(&mut ctx.input_system).button_state_or_default(button_id);

    match state {
        button::ButtonState::Up { .. } => false,
        button::ButtonState::Down { change_frame } => {
            if *change_frame == curr_frame {
                true
            } else {
                false
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ModifierKeyState {
    shift: bool,
    alt: bool,
    ctrl: bool,
    logo: bool,
}

pub(crate) struct InputSystem {
    // TODO: Should I use frame_count or should I instead mutate the state once it is read???
    // I'm wondering whether the second might be better (sth like UP -> DOWN_FIRST -> DOWN_HELD -> UP)
    frame_count: u64,

    cursor_state: cursor::CursorState,
    /// May give incorrect info if cursor_state isn't InsideScreen.
    cursor_pos: cursor::CursorPosition,

    modifier_state: ModifierKeyState,

    /// Includes Mouse buttons, Keyboard keys indexed by scancode and by virtual keys.
    buttons: HashMap<button::ButtonId, button::ButtonState>,
}

impl InputSystem {
    pub(crate) fn new() -> Self {
        Self {
            frame_count: 0,
            cursor_state: cursor::CursorState::Unknown,
            cursor_pos: cursor::CursorPosition {
                x: std::f64::NAN,
                y: std::f64::NAN,
            },
            modifier_state: ModifierKeyState {
                shift: false,
                alt: false,
                ctrl: false,
                logo: false,
            },
            buttons: HashMap::new(),
        }
    }

    // We request &mut self to ensure compatibility if we later decide to go with the mutate the state once it
    // is read approach

    /// Defaults to `ButtonState::Up{change_frame: 0}`
    fn button_state_or_default(&mut self, button_id: &button::ButtonId) -> &button::ButtonState {
        self.buttons
            .get(button_id)
            .unwrap_or(&button::ButtonState::Up { change_frame: 0 })
    }

    /// Since engine event Start (winit event NewEvents) is always the first event emitted in a "frame",
    /// followed by inputs, then only after all inputs is Update (winit event MainEventsCleared) called,
    /// this means it should be safe to update the frame counter at Start and depend on it being the same
    /// for all relevant inputs, and also the same at the time the inputs are processed (assuming the user
    /// calls this module's input reading functions inside `Application::update()`).
    /// This function should therefore be called by the event handler for engine event Start
    /// (see `EventReceiver` impl for `InputSystem`).
    fn start_new_frame(&mut self) {
        self.frame_count += 1;
        //log::trace!("start_new_frame {:?}", self.frame_count);
    }

    fn handle_winit_button_input(
        &mut self,
        button_id: button::ButtonId,
        state: winit::event::ElementState,
    ) {
        if let Some(s) = self.buttons.get(&button_id) {
            match s {
                button::ButtonState::Up { .. } => {
                    if let winit::event::ElementState::Pressed = state {
                        self.buttons.insert(
                            button_id,
                            button::ButtonState::Down {
                                change_frame: self.frame_count,
                            },
                        );
                    }
                }
                button::ButtonState::Down { .. } => {
                    if let winit::event::ElementState::Released = state {
                        self.buttons.insert(
                            button_id,
                            button::ButtonState::Up {
                                change_frame: self.frame_count,
                            },
                        );
                    }
                }
            }
        } else {
            self.buttons.insert(
                button_id,
                match state {
                    winit::event::ElementState::Pressed => button::ButtonState::Down {
                        change_frame: self.frame_count,
                    },
                    winit::event::ElementState::Released => button::ButtonState::Up {
                        change_frame: self.frame_count,
                    },
                },
            );
        };
    }

    pub(crate) fn receive_winit_windowing_event(&mut self, evt: &winit::event::WindowEvent) {
        match evt {
            winit::event::WindowEvent::Resized(_)
            | winit::event::WindowEvent::CloseRequested
            | winit::event::WindowEvent::ScaleFactorChanged { .. }
            | winit::event::WindowEvent::ThemeChanged(_)
            | winit::event::WindowEvent::Moved(_)
            | winit::event::WindowEvent::Destroyed => {
                unreachable!(
                    "Should be filtered out in the main loop, not forwarded to be handled by Input System"
                );
            }
            winit::event::WindowEvent::DroppedFile(_) => {
                // log::warn!("Input System: Currently unsupported event DroppedFile");
            }
            winit::event::WindowEvent::HoveredFile(_) => {
                // log::warn!("Input System: Currently unsupported event HoveredFile");
            }
            winit::event::WindowEvent::HoveredFileCancelled => {
                // log::warn!("Input System: Currently unsupported event HoveredFileCancelled");
            }
            winit::event::WindowEvent::Touch(_) => {
                // log::warn!("Input System: Currently unsupported event Touch");
            }
            // TODO: Should this be handled by input or windowing?
            winit::event::WindowEvent::Focused(_) => {
                // log::warn!("Input System: Currently unsupported event Focused");
            }
            winit::event::WindowEvent::TouchpadPressure { .. } => {
                // log::warn!("Input System: Currently unsupported event TouchpadPressure");
            }
            winit::event::WindowEvent::AxisMotion { .. } => {
                // log::warn!("Input System: Currently unsupported event AxisMotion");
            }
            winit::event::WindowEvent::ReceivedCharacter(_char) => {
                // log::warn!("Input System: Currently unsupported event ReceivedCharacter");
            }
            // The actual interesting code should be here...
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos.x = position.x;
                self.cursor_pos.y = position.y;
            }
            winit::event::WindowEvent::CursorEntered { .. } => {
                self.cursor_state = cursor::CursorState::InsideScreen;
            }
            winit::event::WindowEvent::CursorLeft { .. } => {
                self.cursor_state = cursor::CursorState::OutsideScreen;
            }
            winit::event::WindowEvent::ModifiersChanged(modifiers) => {
                self.modifier_state.shift = modifiers.shift();
                self.modifier_state.ctrl = modifiers.ctrl();
                self.modifier_state.alt = modifiers.alt();
                self.modifier_state.logo = modifiers.logo();
            }

            winit::event::WindowEvent::KeyboardInput {
                input,
                is_synthetic,
                ..
            } => {
                if *is_synthetic {
                    log::warn!("is_synthetic = true for some keyboard event!");
                }

                //----------
                // Handle physical scan code
                let pk = button::ButtonId::KeyboardPhysical(button::KeyboardPhysicalKeyId {
                    scan_code: input.scancode,
                });

                self.handle_winit_button_input(pk, input.state);

                if let Some(winit_virtual_key_code) = input.virtual_keycode {
                    // Handle virtual scan code
                    let vk = button::ButtonId::KeyboardVirtual(button::KeyboardVirtualKeyId {
                        winit_virtual_key_code,
                    });

                    self.handle_winit_button_input(vk, input.state);
                }
            }
            winit::event::WindowEvent::MouseInput { state, button, .. } => {
                let k = self::button::ButtonId::MouseButton(match button {
                    winit::event::MouseButton::Left => self::button::MouseButtonId::Left,
                    winit::event::MouseButton::Right => self::button::MouseButtonId::Right,
                    winit::event::MouseButton::Middle => self::button::MouseButtonId::Middle,
                    winit::event::MouseButton::Other(x) => {
                        self::button::MouseButtonId::Other { winit_id: *x }
                    }
                });

                self.handle_winit_button_input(k, *state);
            }
            winit::event::WindowEvent::MouseWheel { delta, phase, .. } => {
                log::trace!(
                    "TODO handle properly... MouseWheel... delta: {:?}; phase: {:?}",
                    delta,
                    phase
                );
            }
        }
    }

    pub(crate) fn receive_winit_device_event(&mut self, evt: &winit::event::DeviceEvent) {
        log::trace!(
            "Input system received device event but didn't process it: {:?}",
            evt
        );
    }
}

impl crate::event::EventReceiver for InputSystem {
    fn receives_event_type(evt_type: crate::event::types::EventType) -> bool {
        match evt_type {
            crate::event::types::EventType::StartEvent => true,
            _ => false,
        }
    }
    fn receive_event(ctx: &mut crate::EngineContext, evt: crate::event::Event) {
        match evt {
            crate::event::Event::Start => unwrap_mut(&mut ctx.input_system).start_new_frame(),
            _ => unreachable!(),
        }
    }
}
