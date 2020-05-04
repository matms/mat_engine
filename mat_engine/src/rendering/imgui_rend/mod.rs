pub(crate) struct ImguiRenderingSubsystem {
    renderer: imgui_wgpu::Renderer,
}

impl ImguiRenderingSubsystem {
    pub(crate) fn new(
        rendering_system: &mut crate::rendering::RenderingSystem,
        imgui_ctx: &mut imgui::Context,
    ) -> Self {
        let renderer = rendering_system.make_imgui_wgpu_renderer(imgui_ctx);

        Self { renderer }
    }

    pub(crate) fn perform_render(
        &mut self,
        draw_data: &imgui::DrawData,
        rendering_system: &mut crate::rendering::RenderingSystem,
        frt: &mut crate::rendering::FrameRenderTarget,
    ) {
        let encoder = &mut frt.encoder;

        let view = &frt.frame.view;

        let device = &rendering_system.state.device;

        self.renderer
            .render(draw_data, device, encoder, view)
            .expect("Imgui rendering failed");
    }
}
