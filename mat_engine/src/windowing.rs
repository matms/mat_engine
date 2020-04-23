use std::cell::RefCell;
use std::rc::Weak;

// TODO: Refactor into better, more general event system

/// Used to allow systems to be notified whenever the window is resized
pub(crate) trait ResizeListener {
    /// Implement to be notified whenever the window is resized.
    /// Note that we inform the new INNER size of the window. See winit's docs for more
    /// info on the INNER x OUTER size dichotomy.
    fn resize_event(&mut self, new_inner_width: u32, new_inner_height: u32);
}

pub struct WindowingSystem {
    pub(crate) systems: Weak<RefCell<crate::systems::Systems>>,
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
    /// Application::close() will be automatically called, there is no need for
    /// you to call it.
    pub fn force_quit(&mut self) {
        self.force_quit = true;
    }

    pub(crate) fn get_window_ref(&self) -> &winit::window::Window {
        &self.winit_window
    }

    /// Notifies all resize listeners (registered with `add_resize_listener()`, of a resize)
    pub(crate) fn notify_resize(&self, new_inner_width: u32, new_inner_height: u32) {
        let sys_rc = self
            .systems
            .upgrade()
            .expect("Failed to get systems, maybe the Engine has been dropped");
        let systems_ref = sys_rc.borrow();

        if systems_ref.has_rendering() {
            let mut rendering_sys = systems_ref
                .rendering_mut()
                .expect("Failed to borrow Rendering System");
            rendering_sys.resize_event(new_inner_width, new_inner_height);
        }
    }
}

/// Represents a request to the windowing system in the form of an UserEvent.
#[derive(Debug)]
pub enum Request {
    Quit,
}
