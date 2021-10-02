use anyhow::anyhow;
use anyhow::Result as AResult;
use winit::dpi::PhysicalSize;

use super::{
    bind_group::BindGroupable,
    frame::FrameRenderTarget,
    generic_uniform::Uniform,
    wgpu_pipeline::{PipelineBuilder, VertexBufferSetting},
    wgpu_texture::WgpuTexture,
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

    pub(super) window_inner_size: PhysicalSize<u32>,
    pub(super) surface_cfg: wgpu::SurfaceConfiguration,

    // --- ARENAS ---
    // TODO: Maybe move (at least some of) these somewhere else...
    pub(super) textures: Arena<WgpuTexture>,

    pub(super) bind_groups: Arena<BindGroup>,
    pub(super) render_pipelines: Arena<wgpu::RenderPipeline>,
}

impl WgpuState {
    pub(super) fn new(
        // TODO: Abstract -> Remove direct dependency on winit window (see wgpu trait bounds
        // on window)
        window: &winit::window::Window,
    ) -> AResult<Self> {
        let window_inner_size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::all());

        // Safety: window must outlive surface.
        // TODO: Enforce this!
        let surface = unsafe { instance.create_surface(window) };

        let request_adapter_options = &wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            power_preference: wgpu::PowerPreference::default(),
        };

        let adapter: wgpu::Adapter = futures::executor::block_on(async {
            instance.request_adapter(request_adapter_options).await
        })
        .ok_or(anyhow!("adapter could not be obtained"))?;

        let request_device_descriptor = &wgpu::DeviceDescriptor {
            features: wgpu::Features::SPIRV_SHADER_PASSTHROUGH,
            limits: Default::default(),
            label: Some("Default Device"),
        };

        let (device, queue): (wgpu::Device, wgpu::Queue) = futures::executor::block_on(async {
            adapter
                .request_device(request_device_descriptor, None)
                .await
        })?;

        let surface_cfg = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface
                .get_preferred_format(&adapter)
                .ok_or(anyhow!("Preferred format error"))?,
            width: window_inner_size.width,
            height: window_inner_size.height,
            present_mode: wgpu::PresentMode::Fifo, // DOes FIFO work now?
        };

        log::trace!("Preferred format {:#?}", surface_cfg.format);

        surface.configure(&device, &surface_cfg);

        let render_pipelines = Arena::new();
        let textures = Arena::new();
        let bind_groups = Arena::new();

        Ok(Self {
            surface,
            adapter,
            device,
            queue,
            window_inner_size,
            render_pipelines,
            textures,
            bind_groups,
            surface_cfg,
        })
    }

    pub(super) fn resize(&mut self, new_inner_size: PhysicalSize<u32>) {
        log::trace!("Resizing (WgpuState)");
        assert!(new_inner_size.width > 0 && new_inner_size.height > 0);

        self.window_inner_size = new_inner_size;
        self.surface_cfg.width = new_inner_size.width;
        self.surface_cfg.height = new_inner_size.height;

        self.surface.configure(&self.device, &self.surface_cfg);
    }

    /// Returns a `FrameRenderTarget`, which will be used for rendering and must be
    /// given back to complete_frame_render().
    pub(super) fn start_frame_render(&mut self) -> anyhow::Result<FrameRenderTarget> {
        let frame = self.surface.get_current_frame()?;
        let view = frame
            .output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("wgpu renderer encoder"),
            });

        let mut frt = FrameRenderTarget {
            frame,
            view,
            encoder,
        };

        // By default, clear the screen at the start of a frame.
        self.make_clear_render_pass(&mut frt);

        Ok(frt)
    }

    fn make_clear_render_pass<'a>(&'a self, frt: &'a mut FrameRenderTarget) -> RenderPass<'a> {
        let render_pass_descriptor = &wgpu::RenderPassDescriptor {
            label: Some("wgpu render pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &frt.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        };
        let render_pass = frt.encoder.begin_render_pass(render_pass_descriptor);

        RenderPass {
            wgpu_render_pass: render_pass,
        }
    }

    pub(super) fn make_render_pass<'a>(&'a self, frt: &'a mut FrameRenderTarget) -> RenderPass<'a> {
        let render_pass_descriptor = &wgpu::RenderPassDescriptor {
            label: Some("wgpu render pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &frt.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        };
        let render_pass = frt.encoder.begin_render_pass(render_pass_descriptor);

        RenderPass {
            wgpu_render_pass: render_pass,
        }
    }

    pub(super) fn complete_frame_render(&mut self, frt: FrameRenderTarget) {
        self.queue.submit(std::iter::once(frt.encoder.finish()))
    }

    pub(super) fn add_new_render_pipeline(
        &mut self,
        vert_shader: &crate::rendering::shaders::Shader,
        frag_shader: &crate::rendering::shaders::Shader,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
        vertex_buffers: Vec<VertexBufferSetting>,
    ) -> AResult<ArenaKey> {
        let vert_shader_desc = wgpu::ShaderModuleDescriptorSpirV {
            label: Some("Vert Shader"),
            source: vert_shader.as_ref().into(),
        };

        let frag_shader_desc = wgpu::ShaderModuleDescriptorSpirV {
            label: Some("Frag Shader"),
            source: frag_shader.as_ref().into(),
        };

        /*
        let vert_shader_data =
            wgpu::read_spirv(std::io::Cursor::new(vert_shader.as_ref())).unwrap();

        let frag_shader_data =
            wgpu::read_spirv(std::io::Cursor::new(frag_shader.as_ref())).unwrap();
        */

        // Safety: Bad spirV data may cause UB.
        let vert_shader_module =
            unsafe { self.device.create_shader_module_spirv(&vert_shader_desc) };
        let frag_shader_module =
            unsafe { self.device.create_shader_module_spirv(&frag_shader_desc) };

        let render_pipeline_layout =
            self.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("wgpu render pipeline layout"),
                    push_constant_ranges: &[],
                    bind_group_layouts,
                });

        let render_pipeline = PipelineBuilder::new()
            .set_vertex_shader(&vert_shader_module)
            .set_fragment_shader(&frag_shader_module)
            .set_pipeline_layout(&render_pipeline_layout)
            .set_texture_format(self.surface_cfg.format)
            .set_vertex_buffers(vertex_buffers)
            .build(&mut self.device);

        Ok(self.render_pipelines.insert(render_pipeline?))
    }

    /// See `WgpuTexture` for info on the format of the bytes.
    pub(crate) fn add_new_texture_from_bytes(
        &mut self,
        texture_bytes: &[u8],
        label: Option<&'static str>,
    ) -> ArenaKey {
        let (texture, cmd_buf) =
            WgpuTexture::new_from_bytes(&mut self.device, texture_bytes, label).unwrap();

        self.queue.submit(std::iter::once(cmd_buf));

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

        self.queue.submit(std::iter::once(enc.finish()));
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

// TODO: Is it even worth it keeping this type around? Also, If I'm gonna have this,
// shouldn't I also have RenderPipeline wrapped, for consistency?
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
