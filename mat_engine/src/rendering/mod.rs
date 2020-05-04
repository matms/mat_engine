// See https://sotrh.github.io/learn-wgpu/

pub mod frame;
pub mod shaders;

pub(crate) mod imgui_rend;
pub(crate) mod wgpu_state;

use crate::utils::unwrap_mut;
pub use frame::FrameRenderTarget;
use wgpu_state::WgpuState;

pub fn start_render(ctx: &mut crate::EngineContext) -> FrameRenderTarget {
    unwrap_mut(&mut ctx.rendering_system).start_render()
}

pub fn complete_render(ctx: &mut crate::EngineContext, frt: FrameRenderTarget) {
    unwrap_mut(&mut ctx.rendering_system).complete_render(frt);
}

#[allow(dead_code, unused_variables)]
pub struct RenderingSystem {
    pub(crate) state: WgpuState,
}

impl RenderingSystem {
    /// Creates new Rendering System.
    pub(crate) fn new(windowing_system: &crate::windowing::WindowingSystem) -> Self {
        Self {
            state: WgpuState::new(
                windowing_system.get_window_ref(),
                windowing_system.get_window_ref().inner_size().width,
                windowing_system.get_window_ref().inner_size().height,
            ),
        }
    }

    pub(crate) fn start_render(&mut self) -> FrameRenderTarget {
        let mut frt = self.state.start_frame_render();

        // We use a scope here bc we need to borrow frt mutably.
        {
            let mut render_pass = self.state.make_render_pass(&mut frt);

            render_pass.wgpu_render_pass().draw(0..3, 0..1);
        }

        frt
    }

    pub(crate) fn complete_render(&mut self, frt: FrameRenderTarget) {
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
