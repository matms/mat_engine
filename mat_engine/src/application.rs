/// Application Interface
///
/// Unfortunately, winit wants to take control of the main loop. Therefore, we have to manage
/// the general control flow, which is done by requesting an Application "object"
/// (i.e. a struct that impl's Application) and calling it's methods.
/// Nonetheless, most of the engine is still modular, and the user is
/// expected to init most of the engine parts they wish to use.
///
/// All methods are given a `ctx` parameter. This is a handle to the engine, that is
/// needed for various useful procedures. You cannot store this handle (rust lifetime woes),
/// so use the parameter.
///
/// CONTROL FLOW*:
///
/// INIT() -> LOOP { UPDATE() -> RENDER() } -> CLOSE()
///
/// *In general, exceptions/corner-cases exist.
///
/// TODO: Investigate whether to simulate some sort of event_poll() system, or use
/// handle_event.
pub trait Application: super::event::ApplicationEventReceiver {
    /// Creates the Application object.
    ///
    /// Must be implemented by the user
    fn new(ctx: &mut crate::context::EngineContext) -> Self;

    /// Called once the engine wants to close. For example, you may save information here.
    ///
    /// DO NOT CALL DIRECTLY FROM USER CODE, IT WILL NOT CLOSE THE APP. See the `windowing` module
    /// for info on how to cause the application to close.
    #[allow(unused_variables)]
    fn close(&mut self, ctx: &mut crate::context::EngineContext) {}

    /// Called once per frame, after handling events* but before rendering.
    ///
    /// *TODO: Better investigate and document event handling order and corner-cases.
    #[allow(unused_variables)]
    fn update(&mut self, ctx: &mut crate::context::EngineContext) {}

    /// Called once per frame, usually* after `Application::update()`.
    ///
    /// Note: Sometimes, the OS / winit may request a redraw (for example, if the app window is resized),
    /// which could cause `render()` to be called without a preceding `update()` call. You should be careful
    /// to ensure your application functions (reasonably) correctly in this case.
    #[allow(unused_variables)]
    fn render(&mut self, ctx: &mut crate::context::EngineContext) {}
}
