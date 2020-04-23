/// Application Interface
///
/// Unfortunately, winit wants to take control of the main loop. Therefore, we have to manage
/// the general control flow, which is done by requesting an Application "object"
/// (i.e. a struct that impl's Application) and calling it's methods.
/// Nonetheless, most of the engine is still modular, and the user is
/// expected to setup and manage all of the engine parts they wish to use.
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
    fn init(&mut self, engine: &mut crate::systems::Engine) {
        log::trace!("Application initialized. User should probably override Application::init()");
    }

    /// Called once the engine wants to close. For e.g. you may save information here.
    /// DO NOT CALL DIRECTLY FROM USER CODE, IT WILL NOT CLOSE THE APP.
    #[allow(unused_variables)]
    fn close(&mut self, engine: &mut crate::systems::Engine) {
        log::trace!("Application closed");
    }

    /// Called once per frame, after handling events but before rendering
    #[allow(unused_variables)]
    fn update(&mut self, engine: &mut crate::systems::Engine) {}

    /// Called once per frame, after `Application::update()`
    #[allow(unused_variables)]
    fn render(&mut self, engine: &mut crate::systems::Engine) {}

    /// TEMPORARY -> TODO REFACTOR:
    #[allow(unused_variables)]
    fn event_postprocessor(
        &mut self,
        engine: &mut crate::systems::Engine,
        event: &winit::event::Event<crate::windowing::Request>,
    ) {
    }
}
