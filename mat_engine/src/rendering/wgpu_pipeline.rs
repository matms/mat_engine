use anyhow::anyhow;
use anyhow::Result as AResult;

pub(super) struct PipelineBuilder<'a> {
    render_pipeline_layout: Option<&'a wgpu::PipelineLayout>,
    vert_shader_module: Option<&'a wgpu::ShaderModule>,
    frag_shader_module: Option<&'a wgpu::ShaderModule>,
    texture_format: Option<wgpu::TextureFormat>,
    vertex_buffer_settings: Option<Vec<VertexBufferSetting>>,
}

#[allow(dead_code)]
impl<'a> PipelineBuilder<'a> {
    pub(super) fn new() -> Self {
        Self {
            render_pipeline_layout: None,
            vert_shader_module: None,
            frag_shader_module: None,
            texture_format: None,
            vertex_buffer_settings: None,
        }
    }

    pub(super) fn set_pipeline_layout(
        &'a mut self,
        render_pipeline_layout: &'a wgpu::PipelineLayout,
    ) -> &'a mut Self {
        self.render_pipeline_layout = Some(render_pipeline_layout);
        self
    }

    pub(super) fn set_vertex_shader(
        &'a mut self,
        vertex_shader_module: &'a wgpu::ShaderModule,
    ) -> &'a mut Self {
        self.vert_shader_module = Some(vertex_shader_module);
        self
    }

    pub(super) fn set_fragment_shader(
        &'a mut self,
        fragment_shader_module: &'a wgpu::ShaderModule,
    ) -> &'a mut Self {
        self.frag_shader_module = Some(fragment_shader_module);
        self
    }

    pub(super) fn set_texture_format(&'a mut self, format: wgpu::TextureFormat) -> &'a mut Self {
        self.texture_format = Some(format);
        self
    }

    pub(super) fn set_vertex_buffers(
        &'a mut self,
        vertex_buffer_settings: Vec<VertexBufferSetting>,
    ) -> &'a mut Self {
        self.vertex_buffer_settings = Some(vertex_buffer_settings);
        self
    }

    pub(super) fn build(&self, device: &mut wgpu::Device) -> AResult<wgpu::RenderPipeline> {
        let mut vertex_buffer_descriptors = vec![];
        for vbs in self
            .vertex_buffer_settings
            .as_ref()
            .expect("You must set vertex buffer settings")
        {
            vertex_buffer_descriptors.push(wgpu::VertexBufferLayout {
                array_stride: vbs.stride,
                step_mode: vbs.step_mode,
                attributes: vbs.attributes.as_ref(),
            })
        }

        Ok(
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("wgpu render pipeline"),
                layout: Some(
                    self.render_pipeline_layout
                        .ok_or(anyhow!("You must set a layout"))?,
                ),
                vertex: wgpu::VertexState {
                    module: self
                        .vert_shader_module
                        .ok_or(anyhow!("You must set a vertex shader"))?,
                    entry_point: "main",
                    buffers: vertex_buffer_descriptors.as_ref(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: self
                        .frag_shader_module
                        .ok_or(anyhow!("You must set a fragment shader"))?,
                    entry_point: "main",
                    targets: &[wgpu::ColorTargetState {
                        format: self
                            .texture_format
                            .ok_or(anyhow!("You must provide a texture format"))?,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    }],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    clamp_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                }, /*
                   vertex_stage: wgpu::ProgrammableStageDescriptor {
                       module: &self
                           .vert_shader_module
                           .expect("You must set a vertex shader module"),
                       entry_point: "main",
                   },
                   fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                       module: &self
                           .frag_shader_module
                           .expect("You must set a fragment shader module"),
                       entry_point: "main",
                   }),
                   rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                       front_face: wgpu::FrontFace::Ccw,
                       cull_mode: wgpu::CullMode::Back,
                       depth_bias: 0,
                       depth_bias_slope_scale: 0.0,
                       depth_bias_clamp: 0.0,
                   }),
                   primitive_topology: wgpu::PrimitiveTopology::TriangleList, // TODO: What does this do?
                   color_states: &[wgpu::ColorStateDescriptor {
                       format: self
                           .swap_chain_descriptor_format
                           .expect("You must set a swap chain descriptor format"),
                       alpha_blend: wgpu::BlendDescriptor {
                           src_factor: wgpu::BlendFactor::One,
                           dst_factor: wgpu::BlendFactor::One,
                           operation: wgpu::BlendOperation::Add,
                       },
                       color_blend: wgpu::BlendDescriptor {
                           src_factor: wgpu::BlendFactor::SrcAlpha,
                           dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                           operation: wgpu::BlendOperation::Add,
                       },
                       write_mask: wgpu::ColorWrite::ALL,
                   }],
                   depth_stencil_state: None,
                   vertex_state: wgpu::VertexStateDescriptor {
                       index_format: wgpu::IndexFormat::Uint16,
                       vertex_buffers: vertex_buffer_descriptors.as_ref(),
                   },
                   sample_count: 1,
                   sample_mask: !0, // All of them
                   alpha_to_coverage_enabled: false,
                   */
            }),
        )
    }
}

/// Settings corresponding to a single wgpu::VertexBufferDescriptor
pub(super) struct VertexBufferSetting {
    pub stride: wgpu::BufferAddress, // u64
    pub step_mode: wgpu::VertexStepMode,
    pub attributes: Vec<wgpu::VertexAttribute>,
}
