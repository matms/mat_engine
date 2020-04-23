/// ImguiSystem is not a core system, and is not automatically initialized. The user
/// must initialize and manage it.
pub struct ImguiSystem {
    imgui_ctx: ::imgui::Context,
    imgui_winit_platform: imgui_winit_support::WinitPlatform,
    renderer: imgui_wgpu::Renderer,
    render_fns: Vec<Box<dyn FnMut(&mut ::imgui::Ui)>>,
}

impl ImguiSystem {
    pub fn new(
        windowing_system: &mut crate::windowing::WindowingSystem,
        rendering_system: &mut crate::render::RenderingSystem,
    ) -> Self {
        let winit_window = windowing_system.get_window_ref();

        // see https://docs.rs/imgui-winit-support/0.3.1/imgui_winit_support/
        let mut imgui_ctx = ::imgui::Context::create();
        imgui_ctx.set_ini_filename(None);
        let mut imgui_winit_platform = imgui_winit_support::WinitPlatform::init(&mut imgui_ctx);
        imgui_winit_platform.attach_window(
            imgui_ctx.io_mut(),
            winit_window,
            imgui_winit_support::HiDpiMode::Default,
        );

        imgui_ctx
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData {
                config: Some(imgui::FontConfig {
                    ..Default::default()
                }),
            }]);

        let mut renderer = rendering_system.make_imgui_wgpu_renderer(&mut imgui_ctx);

        Self {
            imgui_ctx,
            imgui_winit_platform,
            renderer,
            render_fns: vec![],
        }
    }

    pub fn update(&mut self, windowing_system: &mut crate::windowing::WindowingSystem) {
        let winit_window = windowing_system.get_window_ref();

        self.render_fns.clear();

        self.imgui_winit_platform
            .prepare_frame(self.imgui_ctx.io_mut(), winit_window)
            .expect("Imgui System: Failed to prepare frame");
    }

    pub fn add_render_fn<F>(&mut self, func: F)
    where
        F: 'static,
        F: FnMut(&mut ::imgui::Ui),
    {
        self.render_fns.push(Box::new(func));
    }

    pub fn render(
        &mut self,
        windowing_system: &mut crate::windowing::WindowingSystem,
        rendering_system: &mut crate::render::RenderingSystem,
    ) {
        let mut ui = self.imgui_ctx.frame();

        for f in &mut self.render_fns {
            f(&mut ui);
        }

        self.imgui_winit_platform
            .prepare_render(&ui, windowing_system.get_window_ref());
        // TODO ACTUALLY RENDER
        let draw_data = ui.render();

        {
            let frt = rendering_system.frt.as_mut().unwrap();

            let encoder = &mut frt.encoder;

            let view = &frt.frame.view;

            let device = &rendering_system.state.device;

            self.renderer
                .render(draw_data, device, encoder, view)
                .expect("Imgui rendering failed");
        }
    }

    pub fn process_event(
        &mut self,
        event: &winit::event::Event<crate::windowing::Request>,
        windowing_system: &mut crate::windowing::WindowingSystem,
    ) {
        self.imgui_winit_platform.handle_event(
            self.imgui_ctx.io_mut(),
            windowing_system.get_window_ref(),
            event,
        )
    }
    // TODO handle winit events

    // See https://docs.rs/imgui-winit-support/0.3.1/imgui_winit_support/
    // See https://github.com/unconed/imgui-wgpu-rs/blob/master/examples/hello_world.rs (possibly slightly outdated)
}
