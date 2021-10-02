//! This module provides a default 2d renderer.

pub(crate) mod camera_2d;
pub(crate) mod instance;
pub(crate) mod test_uniform;
pub(crate) mod vertex_2d;

pub mod sprite_renderer;

use super::{
    bind_group::BindGroupable, shaders, vertex_buffer::VertexBufferable, wgpu_state::WgpuState,
    wgpu_texture::WgpuTexture, FrameRenderTarget,
};

use crate::arena::ArenaKey;
use crate::utils::unwrap_mut;
use camera_2d::Camera2d;
use instance::{Instance, InstanceData};
use vertex_2d::Vertex2d;

/// Default 2D renderer component.
///
/// Needs rendering system to be initialized.
#[allow(dead_code)]
pub struct Renderer2d {
    texture_bind_group_layout: wgpu::BindGroupLayout,
    pipeline_key: ArenaKey,
    pub camera: Camera2d,
}

#[allow(dead_code)]
impl Renderer2d {
    pub fn new(ctx: &mut crate::EngineContext) -> Self {
        let wgpu_state = &mut unwrap_mut(&mut ctx.rendering_system).state;

        let tex_desc = WgpuTexture::get_wgpu_bind_group_layout_descriptor();

        let texture_bind_group_layout = wgpu_state.device.create_bind_group_layout(&tex_desc);

        let camera = Camera2d::new(
            wgpu_state.window_inner_width,
            wgpu_state.window_inner_height,
            wgpu_state,
        );

        let pipeline_key = wgpu_state.add_new_render_pipeline(
            rend_2d_vert_shader(),
            rend_2d_frag_shader(),
            &[&texture_bind_group_layout, &camera.camera_bind_group_layout],
            // TODO: Implement another builder for this to allow passing parameters
            vec![
                Vertex2d::buffer_descriptor(0..2), // 0 and 1 -> color and tex coords
                InstanceData::buffer_descriptor(2..6), // 2 through 5 inclusive -> single mat4
            ],
        );

        Self {
            texture_bind_group_layout,
            pipeline_key,
            camera,
        }
    }

    pub fn update(&mut self, ctx: &mut crate::EngineContext) {
        let wgpu_state = &mut unwrap_mut(&mut ctx.rendering_system).state;

        self.camera.feed_screen_size(
            wgpu_state.window_inner_width,
            wgpu_state.window_inner_height,
        );

        self.camera.update(wgpu_state);
    }

    /// You may obtain a new `texture_bind_group_key` by calling `create_new_texture_bind_group()`.
    pub fn render_sample_texture(
        &mut self,
        ctx: &mut crate::EngineContext,
        frt: &mut FrameRenderTarget,
        texture_bind_group_key: ArenaKey,
    ) {
        let wgpu_state = &mut unwrap_mut(&mut ctx.rendering_system).state;

        // BASE DATA: VERTICES, INDICES and INSTANCES

        let vertices = &[
            // A
            Vertex2d {
                position: [-0.5, -0.5],
                tex_coords: [0.0, 1.0],
            },
            // B
            Vertex2d {
                position: [0.5, -0.5],
                tex_coords: [1.0, 1.0],
            },
            // C
            Vertex2d {
                position: [0.5, 0.5],
                tex_coords: [1.0, 0.0],
            },
            // D
            Vertex2d {
                position: [-0.5, 0.5],
                tex_coords: [0.0, 0.0],
            },
        ];

        // See pipeline settings for whether index should be u16 or u32
        let indices: &[u16; 6] = &[
            0, 1, 2, // A B C
            0, 2, 3, // A C D
        ];

        let instances = vec![
            Instance {
                position: nalgebra_glm::vec2(0.0, 0.0),
                scale: 10.0,
            },
            Instance {
                position: nalgebra_glm::vec2(0.0, 30.0),
                scale: 20.0,
            },
            Instance {
                position: nalgebra_glm::vec2(100.0, 100.0),
                scale: 5.0,
            },
        ];

        // BUFFERS

        let vertex_buffer = wgpu_state
            .device
            .create_buffer_with_data(bytemuck::cast_slice(vertices), wgpu::BufferUsage::VERTEX);

        let index_buffer = wgpu_state
            .device
            .create_buffer_with_data(bytemuck::cast_slice(indices), wgpu::BufferUsage::INDEX);

        let instance_data: Vec<InstanceData> = instances.iter().map(Instance::to_data).collect();

        let instance_buffer = wgpu_state.device.create_buffer_with_data(
            bytemuck::cast_slice(&instance_data),
            wgpu::BufferUsage::VERTEX,
        );

        // We use a scope here bc we need to borrow frt mutably.
        {
            let mut render_pass = wgpu_state.make_render_pass(frt);

            render_pass
                .set_pipeline(self.pipeline_key, wgpu_state)
                .unwrap();

            render_pass
                .set_bind_group(0, texture_bind_group_key, &[], wgpu_state)
                .unwrap();

            render_pass
                .set_bind_group(1, self.camera.camera_bind_group_key, &[], wgpu_state)
                .unwrap();

            render_pass
                .wgpu_render_pass
                .set_vertex_buffer(0, &vertex_buffer, 0, 0);

            render_pass
                .wgpu_render_pass
                .set_vertex_buffer(1, &instance_buffer, 0, 0);

            render_pass
                .wgpu_render_pass
                .set_index_buffer(&index_buffer, 0, 0);

            render_pass.wgpu_render_pass.draw_indexed(
                0..(indices.len() as u32),
                0,
                0..(instances.len() as u32),
            );
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

// --- SHADERS ---

lazy_static::lazy_static! {
    static ref COMPILED_DEFAULT_VERT_SHADER: shaders::Shader = {
        unsafe {
            let mut path = crate::assets::get_engine_assets_path();
            path.push("shaders");
            path.push("rend_2d");
            path.push("shader.vert");

            shaders::compile_glsl_to_spirv(
                crate::assets::read_file_at_path_to_string(path).expect("Cannot load shader"),
                "shader.vert".into(),
                shaders::ShaderType::Vertex)
        }
    };

    static ref COMPILED_DEFAULT_FRAG_SHADER: shaders::Shader = {
        unsafe {
            let mut path = crate::assets::get_engine_assets_path();
            path.push("shaders");
            path.push("rend_2d");
            path.push("shader.frag");

            shaders::compile_glsl_to_spirv(
                crate::assets::read_file_at_path_to_string(path).expect("Cannot load shader"),
                "shader.frag".into(),
                shaders::ShaderType::Fragment)
        }
    };
}

fn rend_2d_vert_shader() -> &'static shaders::Shader {
    &*COMPILED_DEFAULT_VERT_SHADER
}

fn rend_2d_frag_shader() -> &'static shaders::Shader {
    &*COMPILED_DEFAULT_FRAG_SHADER
}
