/// Represents the resources necessary to render to screen which were created by
/// `rendering::start_render()`, used as needed, and given back in `rendering::complete_render()`
pub struct FrameRenderTarget {
    pub(crate) frame: wgpu::SwapChainOutput,
    pub(crate) encoder: wgpu::CommandEncoder,
}
