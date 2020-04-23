// See https://sotrh.github.io/learn-wgpu/

use std::{cell::RefCell, rc::Rc};

pub struct RenderingSystem {
    pub(crate) state: WgpuState,
    pub(crate) frt: Option<FrameRenderTarget>,
}

impl RenderingSystem {
    /// Creates new Rendering System.
    ///
    /// Needs mutable borrow of the windowing system because it adds itself as a listener
    /// to certain window events (specifically, for now, it listens to resizes).
    pub(crate) fn new(
        windowing_system: &mut crate::windowing::WindowingSystem,
    ) -> Rc<RefCell<Self>> {
        let out = Rc::new(RefCell::new(Self {
            state: WgpuState::new(
                windowing_system.get_window_ref(),
                windowing_system.get_window_ref().inner_size().width,
                windowing_system.get_window_ref().inner_size().height,
            ),
            frt: None,
        }));
        windowing_system.add_resize_listener(out.clone());
        out
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
}

impl crate::windowing::ResizeListener for RenderingSystem {
    fn resize_event(&mut self, new_inner_width: u32, new_inner_height: u32) {
        self.state.resize(new_inner_width, new_inner_height);
    }
}

/// Do not use directly from user code. It is managed by `RenderingSystem`.
pub(crate) struct WgpuState {
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    pub(crate) device: wgpu::Device,
    queue: wgpu::Queue,
    swap_chain_descriptor: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,

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

        Self {
            surface,
            adapter,
            device,
            queue,
            swap_chain_descriptor,
            swap_chain,
            window_inner_width,
            window_inner_height,
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

        let mut encoder = self
            .device
            .create_command_encoder(command_encoder_descriptor);

        let mut frt = FrameRenderTarget { frame, encoder };

        self.clear_screen(&mut frt);

        frt
    }

    fn complete_render(&mut self, frt: FrameRenderTarget) {
        self.queue.submit(&[frt.encoder.finish()])
    }

    fn clear_screen(&mut self, frt: &mut FrameRenderTarget) {
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
            let _render_pass = frt.encoder.begin_render_pass(render_pass_descriptor);
        }
    }
}

pub struct FrameRenderTarget {
    pub(crate) frame: wgpu::SwapChainOutput,
    pub(crate) encoder: wgpu::CommandEncoder,
}
