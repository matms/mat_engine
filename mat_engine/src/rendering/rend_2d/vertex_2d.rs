use crate::rendering::{
    vertex_buffer::VertexBufferable, vertex_trait::Vertex, wgpu_pipeline::VertexBufferSetting,
};
use std::ops::Range;
use zeroable::Zeroable;

#[repr(C)]
#[derive(Copy, Clone, Debug, Zeroable)]
pub(super) struct Vertex2d {
    pub(crate) position: [f32; 2],
    pub(crate) tex_coords: [f32; 2],
}

// Asserts that there is no padding in Vertex2d
static_assertions::const_assert_eq!(
    std::mem::size_of::<Vertex2d>(),
    std::mem::size_of::<f32>() * 4
);

// Safety:
// See https://docs.rs/bytemuck/1.2.0/bytemuck/trait.Pod.html
// We need to check for the absence of padding, see static assert above.
unsafe impl bytemuck::Pod for Vertex2d {}

impl VertexBufferable for Vertex2d {
    fn buffer_descriptor(shader_locations: Range<u32>) -> VertexBufferSetting {
        let start_shader_location = shader_locations.start;

        assert!(shader_locations.len() == 2);

        VertexBufferSetting {
            stride: std::mem::size_of::<Vertex2d>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: vec![
                wgpu::VertexAttributeDescriptor {
                    // Position
                    offset: 0,
                    format: wgpu::VertexFormat::Float2,
                    shader_location: start_shader_location,
                },
                wgpu::VertexAttributeDescriptor {
                    // Tex Coords
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    format: wgpu::VertexFormat::Float2,
                    shader_location: start_shader_location + 1,
                },
            ],
        }
    }
}

impl Vertex for Vertex2d {}
