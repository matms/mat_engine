use winit::dpi::PhysicalSize;

#[derive(Debug, Copy, Clone)]
pub struct WindowResizeEvent {
    pub new_size: PhysicalSize<u32>,
}
