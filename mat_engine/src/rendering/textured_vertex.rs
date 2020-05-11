use crate::rendering::vertex_trait::Vertex;
use zeroable::Zeroable;

#[repr(C)]
#[derive(Copy, Clone, Debug, Zeroable)]
pub(crate) struct TexturedVertex {
    pub(crate) position: [f32; 3],
    pub(crate) tex_coords: [f32; 2],
}

// Asserts that there is no padding in TexturedVertex
static_assertions::const_assert_eq!(
    std::mem::size_of::<TexturedVertex>(),
    std::mem::size_of::<f32>() * 5
);

// Safety:
// See https://docs.rs/bytemuck/1.2.0/bytemuck/trait.Pod.html
// We need to check for the absence of padding, see static assert above.
unsafe impl bytemuck::Pod for TexturedVertex {}

impl Vertex for TexturedVertex {
    fn buffer_descriptor<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<TexturedVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    // Position
                    offset: 0,
                    format: wgpu::VertexFormat::Float3,
                    shader_location: 0,
                },
                wgpu::VertexAttributeDescriptor {
                    // Tex Coords
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    format: wgpu::VertexFormat::Float2,
                    shader_location: 1,
                },
            ],
        }
    }
}
