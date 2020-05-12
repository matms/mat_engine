use super::{
    bind_group::BindGroupable, textured_vertex::TexturedVertex, wgpu_state::WgpuState,
    FrameRenderTarget,
};
use crate::arena::ArenaKey;
use crate::utils::unwrap_mut;

pub mod wgpu_texture;

/// Default 2D renderer.
///
/// Needs rendering system to be initialized.
#[allow(dead_code)]
pub struct Renderer2d {
    texture_bind_group_layout: wgpu::BindGroupLayout,
    pipeline_key: ArenaKey,
}

#[allow(dead_code)]
impl Renderer2d {
    pub fn new(ctx: &mut crate::EngineContext) -> Self {
        let wgpu_state = &mut unwrap_mut(&mut ctx.rendering_system).state;

        let desc = wgpu_texture::WgpuTexture::get_wgpu_bind_group_layout_descriptor();

        let texture_bind_group_layout = wgpu_state.device.create_bind_group_layout(&desc);

        let pipeline_key = wgpu_state.add_new_render_pipeline::<TexturedVertex>(
            crate::rendering::shaders::default_vert_shader(),
            crate::rendering::shaders::default_frag_shader(),
            &[&texture_bind_group_layout],
        );

        Self {
            texture_bind_group_layout,
            pipeline_key,
        }
    }

    /// You may obtain a new `texture_bind_group_key` by calling `create_new_texture_bind_group()`.
    pub fn render_sample_texture(
        &mut self,
        ctx: &mut crate::EngineContext,
        frt: &mut FrameRenderTarget,
        texture_bind_group_key: ArenaKey,
    ) {
        let wgpu_state = &mut unwrap_mut(&mut ctx.rendering_system).state;

        let vertices = &[
            // A
            TexturedVertex {
                position: [-0.5, -0.5, 0.0],
                tex_coords: [0.0, 1.0],
            },
            // B
            TexturedVertex {
                position: [0.5, -0.5, 0.0],
                tex_coords: [1.0, 1.0],
            },
            // C
            TexturedVertex {
                position: [0.5, 0.5, 0.0],
                tex_coords: [1.0, 0.0],
            },
            // D
            TexturedVertex {
                position: [-0.5, 0.5, 0.0],
                tex_coords: [0.0, 0.0],
            },
        ];

        let vertex_buffer = wgpu_state
            .device
            .create_buffer_with_data(bytemuck::cast_slice(vertices), wgpu::BufferUsage::VERTEX);

        // See pipeline settings for whether index should be u16 or u32
        let indices: &[u16; 6] = &[
            0, 1, 2, // A B C
            0, 2, 3, // A C D
        ];

        let index_buffer = wgpu_state
            .device
            .create_buffer_with_data(bytemuck::cast_slice(indices), wgpu::BufferUsage::INDEX);

        // We use a scope here bc we need to borrow frt mutably.
        {
            let mut render_pass = wgpu_state.make_render_pass(frt);

            render_pass
                .set_pipeline(self.pipeline_key, &wgpu_state)
                .unwrap();

            render_pass
                .set_bind_group(0, texture_bind_group_key, &[], &wgpu_state)
                .unwrap();

            render_pass
                .wgpu_render_pass
                .set_vertex_buffer(0, &vertex_buffer, 0, 0);

            render_pass
                .wgpu_render_pass
                .set_index_buffer(&index_buffer, 0, 0);

            render_pass
                .wgpu_render_pass
                .draw_indexed(0..(indices.len() as u32), 0, 0..1);
        }
    }

    pub fn create_new_texture_bind_group(
        &mut self,
        ctx: &mut crate::EngineContext,
        texture_bytes: &[u8],
        texture_label: Option<&'static str>,
    ) -> ArenaKey {
        let wgpu_state = &mut unwrap_mut(&mut ctx.rendering_system).state;

        let texture_key = self.create_new_texture(wgpu_state, texture_bytes, texture_label);
        let bind_group_key =
            self.create_new_bind_group_from_texture(wgpu_state, texture_key, texture_label);
        bind_group_key
    }

    fn create_new_texture(
        &self,
        wgpu_state: &mut WgpuState,
        texture_bytes: &[u8],
        texture_label: Option<&'static str>,
    ) -> ArenaKey {
        wgpu_state.add_new_texture_from_bytes(texture_bytes, texture_label)
    }

    fn create_new_bind_group_from_texture(
        &self,
        wgpu_state: &mut WgpuState,
        texture_key: ArenaKey,
        texture_label: Option<&'static str>,
    ) -> ArenaKey {
        wgpu_state.add_new_texture_bind_group(
            &self.texture_bind_group_layout,
            texture_key,
            texture_label,
        )
    }
}
