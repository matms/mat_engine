use crate::{imgui::ImguiSystem, rendering::RenderingSystem, windowing::WindowingSystem};
/// "Global" engine state.
///
/// Should probably be used like a singleton. Use of multiple `EngineContexts` is unsupported and untested.
///
/// Systems that aren't initialized by default have init fns to allow users to init them.
///
/// SYSTEMS:
///
/// A - Automatically initialized;
///
/// M - Manual, Need to call `EngineContext::<system_name>_init()`.
///
/// | System    | A/M |
/// |-----------|-----|
/// | windowing |  A  |
/// | rendering |  A  |
/// | imgui     |  M  |
///
/// INTERNAL INFORMATION:
///
/// To access systems conveniently, use `utils::unwrap_ref()` and `utils::unwrap_mut()`
/// Be warned that they panic if the system is not initialized.
pub struct EngineContext {
    pub(crate) windowing_system: Option<WindowingSystem>,
    pub(crate) rendering_system: Option<RenderingSystem>,
    pub(crate) imgui_system: Option<ImguiSystem>,
}

impl EngineContext {
    /// Returns a new, "empty" `EngineContext` with all systems uninitialized.
    pub(crate) fn uninit() -> Self {
        Self {
            windowing_system: None,
            rendering_system: None,
            imgui_system: None,
        }
    }

    /// Automatically called, therefore isn't exported to users of crate.
    pub(crate) fn windowing_init(
        &mut self,
        winit_window: winit::window::Window,
        winit_event_loop_proxy: winit::event_loop::EventLoopProxy<crate::windowing::Request>,
    ) {
        self.windowing_system = Some(crate::windowing::WindowingSystem::new(
            winit_window,
            winit_event_loop_proxy,
        ));
    }

    /// Automatically called, therefore isn't exported to users of crate.
    pub(crate) fn rendering_init(&mut self) {
        self.rendering_system = Some(crate::rendering::RenderingSystem::new(
            &self
                .windowing_system
                .as_mut()
                .expect("Need windowing system to make rendering system"),
        ));
    }

    /// Initialize the Imgui System.
    ///
    /// Needs to be manually called iff the user wishes to use the imgui system, and is therefore
    /// exported to users of the crate.
    ///
    /// Panics if the windowing or rendering systems are uninitialized.
    pub fn imgui_init(&mut self) {
        self.imgui_system = Some(crate::imgui::ImguiSystem::new(
            &self
                .windowing_system
                .as_mut()
                .expect("Need windowing system to make imgui system"),
            &mut self
                .rendering_system
                .as_mut()
                .expect("Need rendering system to make imgui system"),
        ));
    }
}
