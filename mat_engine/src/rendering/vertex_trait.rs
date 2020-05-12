/// Generic Vertex trait
///
/// Note: You must also implement bytemuck::Zeroable (see crate zeroable) and bytemuck::Pod
/// if you want to actually use your vertex, to add it to a buffer.
pub(crate) trait Vertex {
    /// The implementor should return a `VertexBufferDescriptor` that matches the layout of
    /// the struct. Note that the descriptor is generic, any instance of a specific implementor
    /// struct will have the same layout. Therefore, this method need not take in `self`, and
    /// it should return the same thing for any specific implementor type.
    fn buffer_descriptor<'a>() -> wgpu::VertexBufferDescriptor<'a>;
}
