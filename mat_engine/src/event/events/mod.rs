#[derive(Debug, Copy, Clone)]
pub struct WindowResizeEvent {
    pub new_inner_width: u32,
    pub new_inner_height: u32,
}
