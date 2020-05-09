use crate::rendering::vertex_trait::Vertex;
use ::zeroable::Zeroable;

#[repr(C)]
#[derive(Copy, Clone, Debug, Zeroable)]
pub(crate) struct ColoredVertex {
    pub(crate) position: [f32; 3],
    pub(crate) color: [f32; 3],
}

// Asserts that there is no padding in ColoredVertex
static_assertions::const_assert_eq!(
    std::mem::size_of::<ColoredVertex>(),
    std::mem::size_of::<f32>() * 6
);

// Safety:
// See https://docs.rs/bytemuck/1.2.0/bytemuck/trait.Pod.html
// We need to check for the absence of padding, see static assert above.
unsafe impl bytemuck::Pod for ColoredVertex {}

impl Vertex for ColoredVertex {
    fn buffer_descriptor<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<ColoredVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    // Position
                    offset: 0,
                    format: wgpu::VertexFormat::Float3,
                    shader_location: 0,
                },
                wgpu::VertexAttributeDescriptor {
                    // Color
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    format: wgpu::VertexFormat::Float3,
                    shader_location: 1,
                },
            ],
        }
    }
}
