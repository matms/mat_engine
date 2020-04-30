use std::cell::RefCell;
use std::rc::Rc;

/// ImguiSystem is not a core system, and is not automatically initialized. The user
/// must initialize and manage it.
pub struct ImguiSystem {
    pub(crate) systems: Rc<RefCell<crate::systems::Systems>>,
    imgui_ctx: ::imgui::Context,
    imgui_winit_platform: imgui_winit_support::WinitPlatform,
    renderer: imgui_wgpu::Renderer,
    render_fns: Vec<Box<dyn FnMut(&mut ::imgui::Ui)>>,
}

impl ImguiSystem {
    pub fn new(engine: &crate::systems::Engine) -> Self {
        // We take in `Engine` instead of `Rc<RefCell<Systems>>` bc the systems_rc() method is
        // pub(crate), and we don't want to have to expose it. However, to reduce coupling,
        // the only access to engine should be this line. If it is the case that this function is
        // also pub(crate) (i.e. the system is created automatically, by the engine) then the
        // above reason doesn't apply: Instead, we take in Engine for consistency with systems
        // for which the above is the case.
        let systems = engine.systems_rc();

        let systems_ref = systems.borrow();

        let ws = systems_ref.windowing().expect("Failed to Borrow the Windowing System. Note that you must create the Windowing System BEFORE the Imgui System");
        let mut rs = systems_ref.rendering_mut().expect("Failed to Borrow the Rendering System. Note that you must create the Rendering System BEFORE the Imgui System");

        let winit_window = ws.get_window_ref();

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

        let renderer = rs.make_imgui_wgpu_renderer(&mut imgui_ctx);

        Self {
            systems: systems.clone(),
            imgui_ctx,
            imgui_winit_platform,
            renderer,
            render_fns: vec![],
        }
    }

    pub fn update(&mut self) {
        let systems_ref = self.systems.borrow();

        let ws = systems_ref
            .windowing()
            .expect("Failed to Borrow the Windowing System");

        let winit_window = ws.get_window_ref();

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

    pub fn render(&mut self) {
        let systems_ref = self.systems.borrow();

        let ws = systems_ref
            .windowing()
            .expect("Failed to Borrow the Windowing System");

        let mut rs = systems_ref
            .rendering_mut()
            .expect("Failed to Borrow the Windowing System");

        let mut ui = self.imgui_ctx.frame();

        for f in &mut self.render_fns {
            f(&mut ui);
        }

        self.imgui_winit_platform
            .prepare_render(&ui, ws.get_window_ref());
        // TODO ACTUALLY RENDER
        let draw_data = ui.render();

        {
            // Split borrow -> state_and_frt is needed bc. borrowck is stupid.
            let (rs_state, frt) = rs.state_and_frt();

            let frt = frt.as_mut().unwrap();

            let encoder = &mut frt.encoder;

            let view = &frt.frame.view;

            let device = &rs_state.device;

            self.renderer
                .render(draw_data, device, encoder, view)
                .expect("Imgui rendering failed");
        }
    }

    pub fn process_event(&mut self, event: &winit::event::Event<crate::windowing::Request>) {
        let systems_ref = self.systems.borrow();

        let ws = systems_ref
            .windowing()
            .expect("Failed to Borrow the Windowing System");

        self.imgui_winit_platform
            .handle_event(self.imgui_ctx.io_mut(), ws.get_window_ref(), event)
    }

    // See https://docs.rs/imgui-winit-support/0.3.1/imgui_winit_support/
    // See https://github.com/unconed/imgui-wgpu-rs/blob/master/examples/hello_world.rs (possibly slightly outdated)
}
