/// Note: You must also implement bytemuck::Zeroable (see crate zeroable) and bytemuck::Pod
pub(crate) trait Vertex {
    fn buffer_descriptor<'a>() -> wgpu::VertexBufferDescriptor<'a>;
}
