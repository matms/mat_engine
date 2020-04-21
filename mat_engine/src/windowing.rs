pub struct WindowingSystem {
    pub(crate) winit_window: winit::window::Window,
    pub(crate) winit_event_loop_proxy: winit::event_loop::EventLoopProxy<Request>,
    pub(crate) force_quit: bool,
}

impl WindowingSystem {
    /// Sends a quit request to winit's event loop. This will, (possibly after a delay, as
    /// queued events will still be processed), cause the loop to exit. Application::close()
    /// will be automatically called, there is no need for you to call it.
    pub fn queue_quit(&mut self) {
        self.winit_event_loop_proxy
            .send_event(Request::Quit)
            .expect("Couldn't send event to winit event loop.");
    }

    /// Forces winit's event loop to quit, ignoring all outstanding winit events.
    /// Application::close()  will be automatically called, there is no need for
    /// you to call it.
    pub fn force_quit(&mut self) {
        self.force_quit = true;
    }
}

/// Represents a request to the windowing system in the form of an UserEvent.
#[derive(Debug)]
pub(crate) enum Request {
    Quit,
}
