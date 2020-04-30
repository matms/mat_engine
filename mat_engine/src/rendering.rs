// See https://sotrh.github.io/learn-wgpu/

use std::{cell::RefCell, rc::Rc};

#[allow(dead_code, unused_variables)]
pub struct RenderingSystem {
    pub(crate) systems: Rc<RefCell<crate::systems::Systems>>,
    pub(crate) state: WgpuState,
    pub(crate) frt: Option<FrameRenderTarget>,
}

impl RenderingSystem {
    /// Creates new Rendering System.
    ///
    /// Needs mutable borrow of the windowing system because it adds itself as a listener
    /// to certain window events (specifically, for now, it listens to resizes).
    pub(crate) fn new(engine: &crate::systems::Engine) -> Self {
        // We take in `Engine` instead of `Rc<RefCell<Systems>>` bc the systems_rc() method is
        // pub(crate), and we don't want to have to expose it. However, to reduce coupling,
        // the only access to engine should be this line. If it is the case that this function is
        // also pub(crate) (i.e. the system is created automatically, by the engine) then the
        // above reason doesn't apply: Instead, we take in Engine for consistency with systems
        // for which the above is the case.
        let systems = engine.systems_rc();

        let systems_ref = systems.borrow();
        let windowing_system = systems_ref
            .windowing()
            .expect("Failed to Borrow the Windowing System. Note that you must create the Windowing System BEFORE the Rendering System");
        Self {
            systems: systems.clone(),
            state: WgpuState::new(
                windowing_system.get_window_ref(),
                windowing_system.get_window_ref().inner_size().width,
                windowing_system.get_window_ref().inner_size().height,
            ),
            frt: None,
        }
    }

    pub fn start_render(&mut self) {
        self.frt = Some(self.state.start_render());
    }

    pub fn complete_render(&mut self) {
        self.state.complete_render(
            std::mem::replace(&mut self.frt, None)
                .expect("No frame render target: You must've forgotten to call start_render()"),
        );
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
    pub(crate) fn state_and_frt(&mut self) -> (&mut WgpuState, &mut Option<FrameRenderTarget>) {
        return (&mut self.state, &mut self.frt);
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

    render_pipeline: wgpu::RenderPipeline,

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
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
            render_pipeline,
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

    fn start_render(&mut self) -> FrameRenderTarget {
        let frame = self.swap_chain.get_next_texture().unwrap();

        let command_encoder_descriptor = &wgpu::CommandEncoderDescriptor {
            label: Some("wgpu renderer encoder"),
        };

        let encoder = self
            .device
            .create_command_encoder(command_encoder_descriptor);

        let mut frt = FrameRenderTarget { frame, encoder };

        // We use a scope here bc we need to borrow frt mutably.
        {
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
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw(0..3, 0..1);
        }

        // Return frt so other people can render
        frt
    }

    fn complete_render(&mut self, frt: FrameRenderTarget) {
        self.queue.submit(&[frt.encoder.finish()])
    }
}

pub struct FrameRenderTarget {
    pub(crate) frame: wgpu::SwapChainOutput,
    pub(crate) encoder: wgpu::CommandEncoder,
}
