use std::ops::Range;

pub(super) trait VertexBufferable {
    /// The implementor should return a `wgpu_pipeline::VertexBufferSetting` which corresponds to a
    /// `VertexBufferDescriptor` that matches the layout of the struct.
    /// Note that the descriptor is generic, any instance of a specific implementor
    /// struct will have the same layout. Therefore, this method need not take in `self`, and
    /// it should return the same thing for any specific implementor type.
    ///
    /// The user is responsible for managing shader locations. Giving a range of incorrect span _may_
    /// (probably should) panic.
    ///
    /// TODO: Should we also support inclusive range syntax? Probably...
    fn buffer_descriptor(shader_locations: Range<u32>)
        -> super::wgpu_pipeline::VertexBufferSetting;
}
