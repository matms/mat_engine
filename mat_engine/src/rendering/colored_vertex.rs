use super::{vertex_buffer::VertexBufferable, wgpu_pipeline::VertexBufferSetting};
use crate::rendering::vertex_trait::Vertex;
use bytemuck::{Pod, Zeroable};
use std::ops::Range;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub(crate) struct ColoredVertex {
    pub(crate) position: [f32; 3],
    pub(crate) color: [f32; 3],
}

// Asserts that there is no padding in ColoredVertex
static_assertions::const_assert_eq!(
    std::mem::size_of::<ColoredVertex>(),
    std::mem::size_of::<f32>() * 6
);

impl VertexBufferable for ColoredVertex {
    fn buffer_descriptor(shader_locations: Range<u32>) -> VertexBufferSetting {
        let start_shader_location = shader_locations.start;

        assert!(shader_locations.len() == 2);

        VertexBufferSetting {
            stride: std::mem::size_of::<ColoredVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: vec![
                wgpu::VertexAttribute {
                    // Position
                    offset: 0,
                    format: wgpu::VertexFormat::Float32x3,
                    shader_location: start_shader_location,
                },
                wgpu::VertexAttribute {
                    // Color
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    format: wgpu::VertexFormat::Float32x3,
                    shader_location: start_shader_location + 1,
                },
            ],
        }
    }
}

impl Vertex for ColoredVertex {}
