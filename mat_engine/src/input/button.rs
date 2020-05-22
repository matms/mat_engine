#[derive(Debug, Copy, Clone)]
pub enum ButtonState {
    /// Button is up since change_frame
    Up { change_frame: u64 },
    /// Button is down since change_frame
    Down { change_frame: u64 },
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ButtonId {
    MouseButton(MouseButtonId),
    KeyboardPhysical(KeyboardPhysicalKeyId),
    KeyboardVirtual(KeyboardVirtualKeyId),
}

// Useful convenience constants
impl ButtonId {
    pub const MOUSE_LEFT: ButtonId = ButtonId::MouseButton(MouseButtonId::Left);
    pub const MOUSE_RIGHT: ButtonId = ButtonId::MouseButton(MouseButtonId::Right);
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum MouseButtonId {
    Left,
    Right,
    Middle,
    Other { winit_id: u8 },
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct KeyboardPhysicalKeyId {
    pub scan_code: u32,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct KeyboardVirtualKeyId {
    pub winit_virtual_key_code: winit::event::VirtualKeyCode,
}
