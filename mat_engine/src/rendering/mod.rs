// See https://sotrh.github.io/learn-wgpu/

pub mod frame;
pub mod rend_2d;
pub mod shaders;

pub(crate) mod bind_group;
pub(crate) mod colored_vertex;
pub(crate) mod imgui_rend;
pub(crate) mod textured_vertex;
pub(crate) mod vertex_trait;
pub(crate) mod wgpu_state;

use crate::utils::unwrap_mut;
pub use frame::FrameRenderTarget;
use wgpu_state::WgpuState;

/// Starts rendering a new frame. Returns a `FrameRenderTarget` that may be used to draw
/// arbitrary things to screen.
///
/// Wrapper around the `RenderingSystem::start_render()` method.
pub fn start_render(ctx: &mut crate::EngineContext) -> FrameRenderTarget {
    unwrap_mut(&mut ctx.rendering_system).start_render()
}

/// Completes the rendering of a frame. You need to give back ownership of the
/// `FrameRenderTarget` that was created by `start_render()`
///
/// Wrapper around the `RenderingSystem::complete_render()` method.
pub fn complete_render(ctx: &mut crate::EngineContext, frt: FrameRenderTarget) {
    unwrap_mut(&mut ctx.rendering_system).complete_render(frt);
}

/// System that stores state and provides functions related to rendering.
///
/// Currently implemented
pub struct RenderingSystem {
    pub(crate) state: WgpuState,
}

impl RenderingSystem {
    /// Creates a new Rendering System.
    pub(crate) fn new(windowing_system: &crate::windowing::WindowingSystem) -> Self {
        Self {
            state: WgpuState::new(
                windowing_system.get_window_ref(),
                windowing_system.get_window_ref().inner_size().width,
                windowing_system.get_window_ref().inner_size().height,
            ),
        }
    }

    /// See the `start_render()` procedure.
    fn start_render(&mut self) -> FrameRenderTarget {
        let mut frt = self.state.start_frame_render();

        // Clear screen?
        // This may be an issue, with the way we do this.
        // TODO: Investigate.
        self.state.make_render_pass(&mut frt);

        frt
    }

    /// See the `complete_render()` procedure.
    fn complete_render(&mut self, frt: FrameRenderTarget) {
        self.state.complete_frame_render(frt);
    }

    #[cfg(not(feature = "glsl-to-spirv"))]
    pub(crate) fn make_imgui_wgpu_renderer(
        &mut self,
        imgui_ctx: &mut ::imgui::Context,
    ) -> imgui_wgpu::Renderer {
        imgui_wgpu::Renderer::new(
            imgui_ctx,
            &self.state.device,
            &mut self.state.queue,
            self.state.swap_chain_descriptor.format,
            None,
        )
    }

    #[cfg(feature = "glsl-to-spirv")]
    pub(crate) fn make_imgui_wgpu_renderer(
        &mut self,
        imgui_ctx: &mut ::imgui::Context,
    ) -> imgui_wgpu::Renderer {
        imgui_wgpu::Renderer::new_glsl(
            imgui_ctx,
            &self.state.device,
            &mut self.state.queue,
            self.state.swap_chain_descriptor.format,
            None,
        )
    }
}

impl crate::windowing::ResizeListener for RenderingSystem {
    fn resize_event(&mut self, new_inner_width: u32, new_inner_height: u32) {
        self.state.resize(new_inner_width, new_inner_height);
    }
}
