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

/// Utility macro for binding winit virtual keys to our virtual keys
///
///
/// `winit_virtual_key!(MY_LETTER <- A);` expands to:
/// ```
/// pub const MY_LETTER: ButtonId = ButtonId::KeyboardVirtual(KeyboardVirtualKeyId {
///     winit_virtual_key_code: winit::event::VirtualKeyCode::A,
/// });
/// ```
macro_rules! winit_virtual_key {
    ($i: ident <- $p: ident) => {
        pub const $i: ButtonId = ButtonId::KeyboardVirtual(KeyboardVirtualKeyId {
            winit_virtual_key_code: winit::event::VirtualKeyCode::$p,
        });
    };
}

macro_rules! winit_virtual_keys_same_name {
    ($($i: ident),+) => {$(winit_virtual_key!($i <- $i);)*};
}

// Useful convenience constants
// TODO: Implement more.
impl ButtonId {
    pub const MOUSE_LEFT: ButtonId = ButtonId::MouseButton(MouseButtonId::Left);
    pub const MOUSE_RIGHT: ButtonId = ButtonId::MouseButton(MouseButtonId::Right);

    // TODO: Add more keys
    winit_virtual_keys_same_name!(
        A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z
    );

    // TODO: Add more keys
    winit_virtual_key!(LEFT <- Left);
    winit_virtual_key!(UP <- Up);
    winit_virtual_key!(RIGHT <- Right);
    winit_virtual_key!(DOWN <- Down);
    winit_virtual_key!(ESC <- Escape);
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum MouseButtonId {
    Left,
    Right,
    Middle,
    Other { winit_id: u16 },
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct KeyboardPhysicalKeyId {
    pub scan_code: u32,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct KeyboardVirtualKeyId {
    pub winit_virtual_key_code: winit::event::VirtualKeyCode,
}
