// See https://sotrh.github.io/learn-wgpu/

use crate::utils::unwrap_mut;

pub fn start_render(ctx: &mut crate::EngineContext) {
    unwrap_mut(&mut ctx.rendering_context).start_render();
}

pub fn complete_render(ctx: &mut crate::EngineContext) {
    unwrap_mut(&mut ctx.rendering_context).complete_render();
}

enum FrtHolder {
    Owned(FrameRenderTarget),
    Given, // Someone called take_frt() but didn't yet return_frt()
    None,
}

#[allow(dead_code, unused_variables)]
pub struct RenderingSystem {
    pub(crate) state: WgpuState,
    frt: FrtHolder,
}

impl RenderingSystem {
    /// Creates new Rendering System.
    pub(crate) fn new(windowing_context: &crate::windowing::WindowingSystem) -> Self {
        Self {
            state: WgpuState::new(
                windowing_context.get_window_ref(),
                windowing_context.get_window_ref().inner_size().width,
                windowing_context.get_window_ref().inner_size().height,
            ),
            frt: FrtHolder::None,
        }
    }

    pub(crate) fn start_render(&mut self) {
        let mut frt = self.state.start_frame_render();

        // We use a scope here bc we need to borrow frt mutably.
        {
            let mut render_pass = self.state.make_render_pass(&mut frt);

            render_pass.wgpu_render_pass().draw(0..3, 0..1);
        }

        self.frt = FrtHolder::Owned(frt);
    }

    /// DEPRECATED, maybe???
    pub(crate) fn take_frt(&mut self) -> FrameRenderTarget {
        match self.frt {
            FrtHolder::Owned(_) => {
                let x = std::mem::replace(&mut self.frt, FrtHolder::Given);
                if let FrtHolder::Owned(y) = x {
                    y
                } else {
                    unreachable!()
                }
            }
            FrtHolder::Given => {
                log::error!(
                    "Frame render target was taken from Rendering system but not given back yet."
                );
                panic!(
                    "Frame render target was taken from Rendering system but not given back yet."
                );
            }
            FrtHolder::None => {
                log::error!("No frame render target.");
                panic!("No frame render target.");
            }
        }
    }

    pub(crate) fn complete_render(&mut self) {
        match self.frt {
            FrtHolder::Owned(_) => {
                let x = std::mem::replace(&mut self.frt, FrtHolder::None);

                if let FrtHolder::Owned(y) = x {
                    self.state.complete_frame_render(y);
                } else {
                    unreachable!();
                }
            }
            FrtHolder::Given => {
                log::error!(
                    "Frame render target was taken from Rendering system but not given back yet."
                );
                panic!(
                    "Frame render target was taken from Rendering system but not given back yet."
                );
            }
            FrtHolder::None => {
                log::error!("No frame render target.");
                panic!("No frame render target.");
            }
        };
    }

    #[cfg(not(feature = "glsl-to-spirv"))]
    pub(crate) fn make_imgui_wgpu_renderer(
        &mut self,
        imgui_ctx: &mut ::imgui::Context,
    ) -> imgui_wgpu::Renderer {
        imgui_wgpu::Renderer::new(
            imgui_ctx,
            &self.state.device,
            &mut self.state.queue,
            self.state.swap_chain_descriptor.format,
            None,
        )
    }

    #[cfg(feature = "glsl-to-spirv")]
    pub(crate) fn make_imgui_wgpu_renderer(
        &mut self,
        imgui_ctx: &mut ::imgui::Context,
    ) -> imgui_wgpu::Renderer {
        imgui_wgpu::Renderer::new_glsl(
            imgui_ctx,
            &self.state.device,
            &mut self.state.queue,
            self.state.swap_chain_descriptor.format,
            None,
        )
    }

    /// Use to get mutable borrows of both state and frt. Needed bc. borrowck cannot properly
    /// resolve the splitting borrow from other places.
    pub(crate) fn state_and_frt(&mut self) -> (&mut WgpuState, &mut FrameRenderTarget) {
        match &mut self.frt {
            FrtHolder::Owned(x) => {
                return (&mut self.state, x);
            }
            FrtHolder::Given => {
                log::error!(
                    "Frame render target was taken from Rendering system but not given back yet."
                );
                panic!(
                    "Frame render target was taken from Rendering system but not given back yet."
                );
            }
            FrtHolder::None => {
                log::error!("No frame render target.");
                panic!("No frame render target.");
            }
        };
    }
}

impl crate::windowing::ResizeListener for RenderingSystem {
    fn resize_event(&mut self, new_inner_width: u32, new_inner_height: u32) {
        self.state.resize(new_inner_width, new_inner_height);
    }
}

/// Do not use directly from user code. It is managed by `RenderingSystem`.
#[allow(dead_code, unused_variables)]
pub(crate) struct WgpuState {
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    pub(crate) device: wgpu::Device,
    queue: wgpu::Queue,
    swap_chain_descriptor: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,

    default_render_pipeline: wgpu::RenderPipeline,

    window_inner_width: u32,
    window_inner_height: u32,
}

impl WgpuState {
    fn new(
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

        let vert_shader_data = wgpu::read_spirv(std::io::Cursor::new(
            crate::shaders::default_vert_shader().as_ref(),
        ))
        .unwrap();

        let frag_shader_data = wgpu::read_spirv(std::io::Cursor::new(
            crate::shaders::default_frag_shader().as_ref(),
        ))
        .unwrap();

        // TODO: Allow dynamically confinguring shaders.

        let vert_shader_module = device.create_shader_module(&vert_shader_data);
        let frag_shader_module = device.create_shader_module(&frag_shader_data);

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[],
            });

        // https://sotrh.github.io/learn-wgpu/
        // TODO: Allow dynamically creating new pipelines and swapping pipelines
        let default_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                    format: swap_chain_descriptor.format,
                    alpha_blend: wgpu::BlendDescriptor::REPLACE,
                    color_blend: wgpu::BlendDescriptor::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                }],
                depth_stencil_state: None,
                vertex_state: wgpu::VertexStateDescriptor {
                    index_format: wgpu::IndexFormat::Uint16,
                    vertex_buffers: &[],
                },
                sample_count: 1,
                sample_mask: !0, // All of them
                alpha_to_coverage_enabled: false,
            });

        Self {
            surface,
            adapter,
            device,
            queue,
            swap_chain_descriptor,
            swap_chain,
            window_inner_width,
            window_inner_height,
            default_render_pipeline,
        }
    }

    fn resize(&mut self, new_inner_width: u32, new_inner_height: u32) {
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
    fn start_frame_render(&mut self) -> FrameRenderTarget {
        let frame = self.swap_chain.get_next_texture().unwrap();

        let command_encoder_descriptor = &wgpu::CommandEncoderDescriptor {
            label: Some("wgpu renderer encoder"),
        };

        let encoder = self
            .device
            .create_command_encoder(command_encoder_descriptor);

        let frt = FrameRenderTarget { frame, encoder };

        frt
    }

    fn complete_frame_render(&mut self, frt: FrameRenderTarget) {
        self.queue.submit(&[frt.encoder.finish()])
    }

    fn make_render_pass<'a>(&'a self, frt: &'a mut FrameRenderTarget) -> RenderPass<'a> {
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
        let mut render_pass = frt.encoder.begin_render_pass(render_pass_descriptor);

        render_pass.set_pipeline(&self.default_render_pipeline);

        RenderPass {
            wgpu_render_pass: render_pass,
        }
    }
}

pub struct RenderPass<'a> {
    wgpu_render_pass: wgpu::RenderPass<'a>,
}

impl<'a> RenderPass<'a> {
    fn wgpu_render_pass(&mut self) -> &mut wgpu::RenderPass<'a> {
        &mut self.wgpu_render_pass
    }
}

pub struct FrameRenderTarget {
    pub(crate) frame: wgpu::SwapChainOutput,
    pub(crate) encoder: wgpu::CommandEncoder,
}
