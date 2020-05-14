use super::vertex_buffer::VertexBufferable;

/// Generic Vertex trait
///
/// Note: You must also implement bytemuck::Zeroable (see crate zeroable) and bytemuck::Pod
/// if you want to actually use your vertex, to add it to a buffer.
pub(super) trait Vertex: VertexBufferable {}
