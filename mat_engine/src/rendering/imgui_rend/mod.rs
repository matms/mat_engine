pub(crate) struct ImguiRenderingSubsystem {
    renderer: imgui_wgpu::Renderer,
}

impl ImguiRenderingSubsystem {
    pub(crate) fn new(
        rendering_system: &mut crate::rendering::RenderingSystem,
        imgui_ctx: &mut imgui::Context,
    ) -> Self {
        let renderer = rendering_system
            .make_imgui_wgpu_renderer(imgui_ctx, rendering_system.state.surface_cfg.format);

        Self { renderer }
    }

    pub(crate) fn perform_render(
        &mut self,
        draw_data: &imgui::DrawData,
        rendering_system: &mut crate::rendering::RenderingSystem,
        frt: &mut crate::rendering::FrameRenderTarget,
    ) {
        let device = &rendering_system.state.device;
        let queue = &rendering_system.state.queue;

        // TODO: Do we have to do anything else w/ this render_pass?
        let render_pass = &mut rendering_system.state.make_render_pass(frt);

        self.renderer
            .render(draw_data, queue, device, &mut render_pass.wgpu_render_pass)
            .expect("Imgui rendering failed");
    }
}
