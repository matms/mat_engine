use crate::{imgui::ImguiSystem, rendering::RenderingSystem, windowing::WindowingSystem};
/// "Global" engine state
///
/// INTERNAL INFORMATION:
///
/// To access systems conveniently, use `utils::unwrap_ref()` and `utils::unwrap_mut()`
pub struct EngineContext {
    pub(crate) windowing_context: Option<WindowingSystem>,
    pub(crate) rendering_context: Option<RenderingSystem>,
    pub(crate) imgui_context: Option<ImguiSystem>,
}

impl EngineContext {
    pub(crate) fn uninit() -> Self {
        Self {
            windowing_context: None,
            rendering_context: None,
            imgui_context: None,
        }
    }

    /// Systems that aren't initialized by default have init fns to allow users to init them.
    pub fn imgui_init(&mut self) {
        self.imgui_context = Some(crate::imgui::ImguiSystem::new(
            &self
                .windowing_context
                .as_mut()
                .expect("Need windowing context to make imgui context"),
            &mut self
                .rendering_context
                .as_mut()
                .expect("Need rendering context to make imgui context"),
        ));
    }
}
