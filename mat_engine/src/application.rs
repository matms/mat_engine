use std::{cell::RefCell, rc::Rc};

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
    /// The Engine leaves most initialization to the user. Unfortunately,
    /// the windowing system needs to be preinitialized, due to the way winit works.
    /// Therefore, pass initialized_systems to the user, and they may use it as they wish.
    #[allow(unused_variables)]
    fn init(&mut self, initialized_systems: InitializedSystems) {
        log::trace!("Application initialized. User should probably override Application::init()");
    }

    /// Called once the engine wants to close. For e.g. you may save information here.
    /// DO NOT CALL DIRECTLY FROM USER CODE, IT WILL NOT CLOSE THE APP.
    fn close(&mut self) {
        log::trace!("Application closed");
    }

    /// Called once per frame, after handling events but before rendering
    fn update(&mut self) {}

    /// Called once per frame, after `Application::update()`
    fn render(&mut self) {}
}

/// Pointers to systems that are already intialized. To be used by user according to their
/// preferences and goals. Note that if a system is passed by RefCell<>, the user should
/// only access the inner contents when needed (and NOT permanently borrow it).
pub struct InitializedSystems {
    pub windowing_system: Rc<RefCell<crate::windowing::WindowingSystem>>,
    pub rendering_system: Rc<RefCell<crate::render::RenderingSystem>>,
}
