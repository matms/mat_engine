use super::{
    bind_group::BindGroupable, frame::FrameRenderTarget, generic_uniform::Uniform,
    vertex_trait::Vertex, wgpu_texture::WgpuTexture,
};
use crate::{
    arena::{Arena, ArenaKey},
    typedefs::BoxErr,
};

/// Do not use directly from user code. It is managed by `RenderingSystem`.
#[allow(dead_code, unused_variables)]
pub(crate) struct WgpuState {
    pub(super) surface: wgpu::Surface,
    pub(super) adapter: wgpu::Adapter,
    pub(super) device: wgpu::Device,
    pub(super) queue: wgpu::Queue,
    pub(super) swap_chain_descriptor: wgpu::SwapChainDescriptor,
    pub(super) swap_chain: wgpu::SwapChain,

    pub(super) window_inner_width: u32,
    pub(super) window_inner_height: u32,

    // --- ARENAS ---
    // TODO: Maybe move these somewhere else...
    pub(super) textures: Arena<WgpuTexture>,
    pub(super) bind_groups: Arena<BindGroup>,
    pub(super) render_pipelines: Arena<wgpu::RenderPipeline>,
}

impl WgpuState {
    pub(super) fn new(
        // TODO: Abstract -> Remove direct dependency on winit window (see wgpu trait bounds
        // on window)
        window: &winit::window::Window,
        window_inner_width: u32,
        window_inner_height: u32,
    ) -> Self {
        let surface = wgpu::Surface::create(window);

        let request_adapter_options = &wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            power_preference: wgpu::PowerPreference::Default,
        };

        let adapter: wgpu::Adapter = futures::executor::block_on(async {
            wgpu::Adapter::request(
                request_adapter_options,
                // See wgpu docs. As of writing: Vulkan, Metal and DX12 are PRIMARY.
                wgpu::BackendBit::PRIMARY,
            )
            .await
            .unwrap()
        });

        let request_device_descriptor = &wgpu::DeviceDescriptor {
            extensions: wgpu::Extensions {
                anisotropic_filtering: false,
            },
            limits: Default::default(),
        };

        let (device, queue): (wgpu::Device, wgpu::Queue) = futures::executor::block_on(async {
            adapter.request_device(request_device_descriptor).await
        });

        let swap_chain_descriptor = wgpu::SwapChainDescriptor {
            // Texture will be used to output to screen
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            // Bgra8UnormSrgb should be supported by all APIs.
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: window_inner_width,
            height: window_inner_height,
            // See https://docs.rs/wgpu/0.5.0/wgpu/enum.PresentMode.html
            // Note: It seems that Immediate is not working on my PC, as it is
            // falling back to Fifo, as the docs describe.
            // Mailbox seems to induce some frames having longer frametimes than other,
            // I should investigate.
            // Fifo just means wait for Vsync.
            present_mode: wgpu::PresentMode::Fifo,
        };

        let swap_chain = device.create_swap_chain(&surface, &swap_chain_descriptor);

        let render_pipelines = Arena::new();
        let textures = Arena::new();
        let bind_groups = Arena::new();

        Self {
            surface,
            adapter,
            device,
            queue,
            swap_chain_descriptor,
            swap_chain,
            window_inner_width,
            window_inner_height,
            render_pipelines,
            textures,
            bind_groups,
        }
    }

    pub(super) fn resize(&mut self, new_inner_width: u32, new_inner_height: u32) {
        log::trace!("Resizing (WgpuState)");
        self.window_inner_width = new_inner_width;
        self.window_inner_height = new_inner_height;
        self.swap_chain_descriptor.width = new_inner_width;
        self.swap_chain_descriptor.height = new_inner_height;
        self.swap_chain = self
            .device
            .create_swap_chain(&self.surface, &self.swap_chain_descriptor);
    }

    /// Returns a `FrameRenderTarget`, which will be used for rendering and must be
    /// given back to complete_frame_render().
    pub(super) fn start_frame_render(&mut self) -> FrameRenderTarget {
        let frame = self.swap_chain.get_next_texture().unwrap();

        let encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("wgpu renderer encoder"),
            });

        FrameRenderTarget { frame, encoder }
    }

    pub(super) fn complete_frame_render(&mut self, frt: FrameRenderTarget) {
        self.queue.submit(&[frt.encoder.finish()])
    }

    pub(super) fn make_render_pass<'a>(&'a self, frt: &'a mut FrameRenderTarget) -> RenderPass<'a> {
        let render_pass_descriptor = &wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frt.frame.view,
                resolve_target: None,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: wgpu::Color {
                    r: 0.1,
                    g: 0.2,
                    b: 0.3,
                    a: 1.0,
                },
            }],
            depth_stencil_attachment: None,
        };
        let render_pass = frt.encoder.begin_render_pass(render_pass_descriptor);

        RenderPass {
            wgpu_render_pass: render_pass,
        }
    }

    pub(super) fn add_new_render_pipeline<T>(
        &mut self,
        vert_shader: &crate::rendering::shaders::Shader,
        frag_shader: &crate::rendering::shaders::Shader,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
    ) -> ArenaKey
    where
        T: Vertex,
    {
        log::trace!(
            "Creating render pipeline using vertex type `{}`, make sure that you are passing in the correct vertex type.",
            std::any::type_name::<T>(),
        );

        let vert_shader_data =
            wgpu::read_spirv(std::io::Cursor::new(vert_shader.as_ref())).unwrap();

        let frag_shader_data =
            wgpu::read_spirv(std::io::Cursor::new(frag_shader.as_ref())).unwrap();

        let vert_shader_module = self.device.create_shader_module(&vert_shader_data);
        let frag_shader_module = self.device.create_shader_module(&frag_shader_data);

        let render_pipeline_layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { bind_group_layouts });

        // https://sotrh.github.io/learn-wgpu/
        // TODO: Allow dynamically creating new pipelines and swapping pipelines
        let render_pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                layout: &render_pipeline_layout,
                vertex_stage: wgpu::ProgrammableStageDescriptor {
                    module: &vert_shader_module,
                    entry_point: "main",
                },
                fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                    module: &frag_shader_module,
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
                    format: self.swap_chain_descriptor.format,
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
                    vertex_buffers: &[
                        // Todo: Allow dynamically changing this
                        T::buffer_descriptor(),
                    ],
                },
                sample_count: 1,
                sample_mask: !0, // All of them
                alpha_to_coverage_enabled: false,
            });

        self.render_pipelines.insert(render_pipeline)
    }

    /// See `WgpuTexture` for info on the format of the bytes.
    pub(crate) fn add_new_texture_from_bytes(
        &mut self,
        texture_bytes: &[u8],
        label: Option<&'static str>,
    ) -> ArenaKey {
        let (texture, cmd_buf) =
            WgpuTexture::new_from_bytes(&mut self.device, texture_bytes, label).unwrap();

        self.queue.submit(&[cmd_buf]);

        self.textures.insert(texture)
    }

    pub(crate) fn add_new_texture_bind_group(
        &mut self,
        bind_group_layout: &wgpu::BindGroupLayout,
        texture_key: ArenaKey,
        label: Option<&'static str>,
    ) -> ArenaKey {
        let wgpu_bind_group = self
            .textures
            .get(texture_key)
            .expect("The texture doesn't exist")
            .make_wgpu_bind_group(bind_group_layout, &mut self.device);

        self.bind_groups.insert(BindGroup {
            wgpu_bind_group,
            label,
        })
    }

    /// Analogous to `add_new_texture_bind_group()`, but for uniforms.
    /// Note that while we currently (as of the time I wrote this, but it may change) store
    /// textures inside WgpuState, we do NOT store uniforms in WgpuState, so we pass in
    /// the uniform itself and not a key, which is the main difference between
    /// `add_new_texture_bind_group()` and `add_new_uniform_bind_group()`
    pub(super) fn add_new_uniform_bind_group<T: Uniform>(
        &mut self,
        bind_group_layout: &wgpu::BindGroupLayout,
        uniform: &T,
        label: Option<&'static str>,
    ) -> ArenaKey {
        let wgpu_bind_group = uniform.make_wgpu_bind_group(bind_group_layout, &mut self.device);

        self.bind_groups.insert(BindGroup {
            wgpu_bind_group,
            label,
        })
    }

    /// Updates a single uniform's buffer using it's `Uniform::update_buffer()` method.
    /// The buffer is stored inside the uniform.
    ///
    /// See `wgpu_generic_uniform::Uniform` and the `Uniform::update_buffer()` method for specific details.
    ///
    /// Note: We create a new command encoder every time this is called, and we submit to the queue
    /// every time as well. I'm not sure if this is a possible performance issue.
    pub(super) fn update_uniform_buffer<T: Uniform>(&mut self, uniform: &T) {
        let mut enc = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("test_uniform_update_encoder"),
            });

        uniform.update_buffer(&mut enc, &mut self.device);

        self.queue.submit(&[enc.finish()]);
    }
}

pub(crate) struct RenderPass<'a> {
    pub(super) wgpu_render_pass: wgpu::RenderPass<'a>,
}

impl<'a> RenderPass<'a> {
    /// Wrapper around `wgpu::RenderPass::set_bind_group()` but using an `ArenaKey`
    /// to index into the bind group arena located in `WgpuState`.
    pub(crate) fn set_bind_group(
        &mut self,
        index: u32,
        bind_group_key: ArenaKey,
        offsets: &[wgpu::DynamicOffset],
        wgpu_state: &'a WgpuState,
    ) -> Result<(), BoxErr> {
        let bind_group = wgpu_state
            .bind_groups
            .get(bind_group_key)
            .ok_or("Bind group doesn't exist")?;

        self.wgpu_render_pass
            .set_bind_group(index, &bind_group.wgpu_bind_group, offsets);

        Ok(())
    }

    /// Wrapper around `wgpu::RenderPass::set_pipeline()` but using an `ArenaKey`
    /// to index into the pipeline arena located in `WgpuState`.
    pub(super) fn set_pipeline(
        &mut self,
        pipeline_key: ArenaKey,
        wgpu_state: &'a WgpuState,
    ) -> Result<(), crate::typedefs::BoxErr> {
        let pipeline = wgpu_state
            .render_pipelines
            .get(pipeline_key)
            .ok_or("Pipeline doesn't exist")?;

        self.wgpu_render_pass.set_pipeline(pipeline);

        Ok(())
    }
}

pub(crate) struct BindGroup {
    pub(super) wgpu_bind_group: wgpu::BindGroup,
    #[allow(dead_code)]
    pub(super) label: Option<&'static str>,
}

/*
impl BindGroup {
    pub(crate) fn wgpu_bind_group(&mut self) -> &mut wgpu::BindGroup {
        &mut self.wgpu_bind_group
    }
}
*/
