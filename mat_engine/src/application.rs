/// Application Interface
///
/// Unfortunately, winit wants to take control of the main loop. Therefore, we have to manage
/// the general control flow, which is done by requesting an Application "object"
/// (i.e. a struct that impl's Application) and calling it's methods.
/// Nonetheless, most of the engine is still modular, and the user is
/// expected to init most of the engine parts they wish to use.
///
/// CONTROL FLOW:
///
/// INIT -> LOOP { UPDATE -> RENDER } -> CLOSE
///
/// TODO: Investigate whether to simulate some sort of event_poll() system, or use
/// handle_event.
pub trait Application {
    /// Called once, at initialization.
    #[allow(unused_variables)]
    fn init(&mut self, ctx: &mut crate::context::EngineContext) {}

    /// Called once the engine wants to close. For example, you may save information here.
    ///
    /// DO NOT CALL DIRECTLY FROM USER CODE, IT WILL NOT CLOSE THE APP. See the `windowing` module
    /// for info on how to cause the application to close.
    #[allow(unused_variables)]
    fn close(&mut self, ctx: &mut crate::context::EngineContext) {}

    /// Called once per frame, after handling events but before rendering.
    #[allow(unused_variables)]
    fn update(&mut self, ctx: &mut crate::context::EngineContext) {}

    /// Called once per frame, after `Application::update()`.
    #[allow(unused_variables)]
    fn render(&mut self, ctx: &mut crate::context::EngineContext) {}
}
