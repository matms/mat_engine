/// Represents the resources necessary to render a frame to screen, which are created by
/// `rendering::start_render()`, used as needed, including possibly passing to other systems
/// (the imgui system, for example, needs to mutably borrow a `FrameRenderTarget` to be
/// able to actually proceed with rendering to screen), and finally given back in
/// `rendering::complete_render()` in order to complete the rendering.
pub struct FrameRenderTarget {
    pub(super) frame: wgpu::SwapChainOutput,
    pub(super) encoder: wgpu::CommandEncoder,
}
