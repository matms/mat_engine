/// Any type that wishes to provide data associated with it to the GPU through usage of bind groups
/// should implement this trait.
///
/// Important note: The data provided to the GPU need not be the contents of the implementor struct
/// itself. Indeed, it is likely that the implementor struct contains one or more members which are
/// what is actually provided to the GPU. The data should be stored in the struct and not somewhere
/// else because in the latter case that would make implementing `make_wgpu_bind_group()` very
/// challenging.
pub(super) trait BindGroupable {
    /// General -> `wgpu::BindGroupLayoutDescriptor` is the same for any specific implementor type.
    ///
    /// Note that we therefore do not take in self.
    fn get_wgpu_bind_group_layout_descriptor() -> wgpu::BindGroupLayoutDescriptor<'static>;

    /// Specific -> Each instance of `Self` returns a potentially different `wgpu::BindGroup`
    ///
    /// This method likely depends on data located in self, such as a wgpu buffer or a texture view,
    /// for example.
    fn make_wgpu_bind_group(
        &self,
        bind_group_layout: &wgpu::BindGroupLayout,
        device: &mut wgpu::Device,
    ) -> wgpu::BindGroup;
}
