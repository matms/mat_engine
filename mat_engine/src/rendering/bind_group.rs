/// Any type that wishes to provide data associated with it to the GPU through usage of bind groups
/// should implement this trait.
pub(super) trait BindGroupable {
    /// General -> `wgpu::BindGroupLayoutDescriptor` is the same for any specific type.
    fn get_wgpu_bind_group_layout_descriptor() -> wgpu::BindGroupLayoutDescriptor<'static>;

    /// Specific -> Each instance of `Self` returns a potentially different `wgpu::BindGroup`
    fn make_wgpu_bind_group<'a>(
        &self,
        bind_group_layout: &wgpu::BindGroupLayout,
        device: &mut wgpu::Device,
    ) -> wgpu::BindGroup;
}
