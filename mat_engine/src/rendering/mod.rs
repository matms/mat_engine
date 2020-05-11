// See https://sotrh.github.io/learn-wgpu/

pub mod frame;
pub mod shaders;

pub(crate) mod colored_vertex;
pub(crate) mod imgui_rend;
pub(crate) mod rend_2d;
pub(crate) mod textured_vertex;
pub(crate) mod vertex_trait;
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
            let vertices = &[
                // A
                textured_vertex::TexturedVertex {
                    position: [-0.5, -0.5, 0.0],
                    tex_coords: [0.0, 1.0],
                },
                // B
                textured_vertex::TexturedVertex {
                    position: [0.5, -0.5, 0.0],
                    tex_coords: [1.0, 1.0],
                },
                // C
                textured_vertex::TexturedVertex {
                    position: [0.5, 0.5, 0.0],
                    tex_coords: [1.0, 0.0],
                },
                // D
                textured_vertex::TexturedVertex {
                    position: [-0.5, 0.5, 0.0],
                    tex_coords: [0.0, 0.0],
                },
            ];

            let vertex_buffer = self
                .state
                .device
                .create_buffer_with_data(bytemuck::cast_slice(vertices), wgpu::BufferUsage::VERTEX);

            // See pipeline settings for whether index should be u16 or u32
            let indices: &[u16; 6] = &[
                0, 1, 2, // A B C
                0, 2, 3, // A C D
            ];

            let index_buffer = self
                .state
                .device
                .create_buffer_with_data(bytemuck::cast_slice(indices), wgpu::BufferUsage::INDEX);

            let mut render_pass = self.state.make_render_pass(&mut frt);

            // Set default pipeline
            self.state
                .set_render_pass_pipeline(&mut render_pass, self.state.default_render_pipeline_key)
                .expect("Failed to set default pipeline, maybe it doesn't exist");

            render_pass.wgpu_render_pass().set_bind_group(
                0,
                &self
                    .state
                    .bind_groups
                    .get(self.state.default_bind_group_key)
                    .expect("No default bind group, for some reason"),
                &[],
            );

            render_pass
                .wgpu_render_pass()
                .set_vertex_buffer(0, &vertex_buffer, 0, 0);

            render_pass
                .wgpu_render_pass()
                .set_index_buffer(&index_buffer, 0, 0);

            render_pass
                .wgpu_render_pass()
                .draw_indexed(0..(indices.len() as u32), 0, 0..1);
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
