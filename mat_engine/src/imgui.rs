use crate::{
    rendering::imgui_rend::ImguiRenderingSubsystem,
    utils::{unwrap_mut, unwrap_ref},
};

use std::sync::{Mutex, MutexGuard};

const USE_GLOBAL_DEBUG_RENDER_FNS: bool = true;

lazy_static::lazy_static! {
    static ref GLOBAL_DEBUG_RENDER_FNS: Mutex<Vec<Box<dyn FnMut(&mut ::imgui::Ui) + Send>>> =
        Mutex::new(vec![]);
}

pub fn global_debug_add_render_fn<F>(func: F)
where
    F: 'static,
    F: FnMut(&mut ::imgui::Ui),
    F: Send,
{
    if USE_GLOBAL_DEBUG_RENDER_FNS {
        GLOBAL_DEBUG_RENDER_FNS.lock().expect(
            "Failed to lock GLOBAL_DEBUG_RENDER_FNS Mutex. I should probably do something about this",
        ).push(Box::new(func));
    } else {
        log::warn!(
            "Attempted to add global debug render fn, but USE_GLOBAL_DEBUG_RENDER_FNS is false."
        );
    }
}

pub fn update(ctx: &mut crate::EngineContext) {
    unwrap_mut(&mut ctx.imgui_system).update(unwrap_ref(&mut ctx.windowing_system));
}

pub fn add_render_fn<F>(ctx: &mut crate::EngineContext, func: F)
where
    F: 'static,
    F: FnMut(&mut ::imgui::Ui),
{
    unwrap_mut(&mut ctx.imgui_system).add_render_fn(func);
}

pub fn render(ctx: &mut crate::EngineContext, frt: &mut crate::rendering::FrameRenderTarget) {
    unwrap_mut(&mut ctx.imgui_system).render(
        unwrap_ref(&ctx.windowing_system),
        unwrap_mut(&mut ctx.rendering_system),
        frt,
    );
}

pub(crate) fn process_event(
    ctx: &mut crate::EngineContext,
    event: &winit::event::Event<crate::windowing::Request>,
) {
    unwrap_mut(&mut ctx.imgui_system).process_event(unwrap_ref(&ctx.windowing_system), event);
}

/// ImguiSystem is not a core system, and is not automatically initialized. The user
/// must initialize and manage it.
pub struct ImguiSystem {
    imgui_ctx: ::imgui::Context,
    imgui_winit_platform: imgui_winit_support::WinitPlatform,
    rendering_subsystem: ImguiRenderingSubsystem,
    render_fns: Vec<Box<dyn FnMut(&mut ::imgui::Ui)>>,
}

impl ImguiSystem {
    pub(crate) fn new(
        windowing_system: &crate::windowing::WindowingSystem,
        rendering_system: &mut crate::rendering::RenderingSystem,
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

        let rendering_subsystem = ImguiRenderingSubsystem::new(rendering_system, &mut imgui_ctx);

        Self {
            imgui_ctx,
            imgui_winit_platform,
            rendering_subsystem,
            render_fns: vec![],
        }
    }

    pub(crate) fn update(&mut self, windowing_system: &crate::windowing::WindowingSystem) {
        let winit_window = windowing_system.get_window_ref();

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
        windowing_system: &crate::windowing::WindowingSystem,
        rendering_system: &mut crate::rendering::RenderingSystem,
        frt: &mut crate::rendering::FrameRenderTarget,
    ) {
        let mut ui = self.imgui_ctx.frame();

        for f in &mut self.render_fns {
            f(&mut ui);
        }

        if USE_GLOBAL_DEBUG_RENDER_FNS {
            let mut fs: MutexGuard<Vec<Box<dyn FnMut(&mut ::imgui::Ui) + Send>>>  = GLOBAL_DEBUG_RENDER_FNS.lock().expect(
                "Failed to lock GLOBAL_DEBUG_RENDER_FNS Mutex. I should probably do something about this"
            );

            for f in &mut fs.iter_mut() {
                f(&mut ui);
            }

            fs.clear();
        }

        self.imgui_winit_platform
            .prepare_render(&ui, windowing_system.get_window_ref());

        let draw_data = ui.render();

        self.rendering_subsystem
            .perform_render(draw_data, rendering_system, frt);

        self.render_fns.clear();
    }

    pub(crate) fn process_event(
        &mut self,
        windowing_system: &crate::windowing::WindowingSystem,
        event: &winit::event::Event<crate::windowing::Request>,
    ) {
        self.imgui_winit_platform.handle_event(
            self.imgui_ctx.io_mut(),
            windowing_system.get_window_ref(),
            event,
        )
    }

    // See https://docs.rs/imgui-winit-support/0.3.1/imgui_winit_support/
    // See https://github.com/unconed/imgui-wgpu-rs/blob/master/examples/hello_world.rs (possibly slightly outdated)
}
