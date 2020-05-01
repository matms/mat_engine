use crate::utils::{unwrap_mut, unwrap_ref};

pub fn update(ctx: &mut crate::EngineContext) {
    unwrap_mut(&mut ctx.imgui_context).update(unwrap_ref(&mut ctx.windowing_context));
}

pub fn add_render_fn<F>(ctx: &mut crate::EngineContext, func: F)
where
    F: 'static,
    F: FnMut(&mut ::imgui::Ui),
{
    unwrap_mut(&mut ctx.imgui_context).add_render_fn(func);
}

pub fn render(ctx: &mut crate::EngineContext) {
    unwrap_mut(&mut ctx.imgui_context).render(
        unwrap_ref(&ctx.windowing_context),
        unwrap_mut(&mut ctx.rendering_context),
    );
}

pub fn process_event(
    ctx: &mut crate::EngineContext,
    event: &winit::event::Event<crate::windowing::Request>,
) {
    unwrap_mut(&mut ctx.imgui_context).process_event(unwrap_ref(&ctx.windowing_context), event);
}

/// ImguiSystem is not a core system, and is not automatically initialized. The user
/// must initialize and manage it.
pub struct ImguiSystem {
    imgui_ctx: ::imgui::Context,
    imgui_winit_platform: imgui_winit_support::WinitPlatform,
    renderer: imgui_wgpu::Renderer,
    render_fns: Vec<Box<dyn FnMut(&mut ::imgui::Ui)>>,
}

impl ImguiSystem {
    pub(crate) fn new(
        windowing_context: &crate::windowing::WindowingSystem,
        rendering_context: &mut crate::rendering::RenderingSystem,
    ) -> Self {
        let winit_window = windowing_context.get_window_ref();

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

        let renderer = rendering_context.make_imgui_wgpu_renderer(&mut imgui_ctx);

        Self {
            imgui_ctx,
            imgui_winit_platform,
            renderer,
            render_fns: vec![],
        }
    }

    pub(crate) fn update(&mut self, windowing_context: &crate::windowing::WindowingSystem) {
        let winit_window = windowing_context.get_window_ref();

        self.render_fns.clear();

        self.imgui_winit_platform
            .prepare_frame(self.imgui_ctx.io_mut(), winit_window)
            .expect("Imgui System: Failed to prepare frame");
    }

    pub(crate) fn add_render_fn<F>(&mut self, func: F)
    where
        F: 'static,
        F: FnMut(&mut ::imgui::Ui),
    {
        self.render_fns.push(Box::new(func));
    }

    pub(crate) fn render(
        &mut self,
        windowing_context: &crate::windowing::WindowingSystem,
        rendering_context: &mut crate::rendering::RenderingSystem,
    ) {
        let mut ui = self.imgui_ctx.frame();

        for f in &mut self.render_fns {
            f(&mut ui);
        }

        self.imgui_winit_platform
            .prepare_render(&ui, windowing_context.get_window_ref());

        let draw_data = ui.render();

        {
            // Split borrow -> state_and_frt is needed bc. borrowck is stupid.
            let (rs_state, frt) = rendering_context.state_and_frt();

            let encoder = &mut frt.encoder;

            let view = &frt.frame.view;

            let device = &rs_state.device;

            self.renderer
                .render(draw_data, device, encoder, view)
                .expect("Imgui rendering failed");
        }
    }

    pub(crate) fn process_event(
        &mut self,
        windowing_context: &crate::windowing::WindowingSystem,
        event: &winit::event::Event<crate::windowing::Request>,
    ) {
        self.imgui_winit_platform.handle_event(
            self.imgui_ctx.io_mut(),
            windowing_context.get_window_ref(),
            event,
        )
    }

    // See https://docs.rs/imgui-winit-support/0.3.1/imgui_winit_support/
    // See https://github.com/unconed/imgui-wgpu-rs/blob/master/examples/hello_world.rs (possibly slightly outdated)
}
