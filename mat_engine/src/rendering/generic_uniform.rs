use super::bind_group::BindGroupable;

/// Implementations of `Uniform` are expected to store both their content (see associated type `Content`)
/// and also their buffer (`wgpu::Buffer`).
///
/// Note that you must also implement `BindGroupable`, as uniforms are actually used with bind groups.
pub(super) trait Uniform: BindGroupable {
    type Content;

    /// This should be called once, likely in the implementor struct `new()` method.
    ///
    /// This generates a wgpu buffer containing the contents of `content`, which the implementor struct
    /// should store alongside `content: Content` itself.
    ///
    ///
    /// TODO: Investigate whether this is needed as a part of the trait. (After all, the
    /// function who calls this, which is probably `new()`, is likely in the same file as this method's
    /// implementation).
    fn create_new_buffer(
        content: Self::Content,
        device: &mut wgpu::Device,
        usage: wgpu::BufferUsage,
    ) -> wgpu::Buffer;

    /// Uses the encoder passed in to update the buffer stored in the uniform object.
    ///
    /// Remember to actually submit the encoder to the queue, otherwise nothing will change.
    ///
    /// This is called by `WgpuState::update_uniform_buffer()`, which creates the encoder and submits
    /// it to the queue as described above.
    fn update_buffer(&self, encoder: &mut wgpu::CommandEncoder, device: &mut wgpu::Device);
}
