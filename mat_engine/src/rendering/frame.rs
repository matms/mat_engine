/// Represents the resources necessary to render a frame to screen, which are created by
/// `rendering::start_render()`, used as needed, including possibly passing to other systems
/// (the imgui system, for example, needs to mutably borrow a `FrameRenderTarget` to be
/// able to actually proceed with rendering to screen), and finally given back in
/// `rendering::complete_render()` in order to complete the rendering.
pub struct FrameRenderTarget {
    // Note: Due to issue https://github.com/gfx-rs/wgpu/issues/1797, view must be dropped before frame.
    // So the order of declaration here is important.
    // If you declare frame first, then view second, this will cause a runtime crash.
    pub(super) view: wgpu::TextureView,
    pub(super) frame: wgpu::SurfaceFrame,
    pub(super) encoder: wgpu::CommandEncoder,
}
