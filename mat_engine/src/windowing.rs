use std::{cell::RefCell, rc::Rc};

// TODO: Refactor into better, more general event system

/// Used to allow systems to be notified whenever the window is resized
pub(crate) trait ResizeListener {
    /// Implement to be notified whenever the window is resized.
    /// Note that we inform the new INNER size of the window. See winit's docs for more
    /// info on the INNER x OUTER size dichotomy.
    fn resize_event(&mut self, new_inner_width: u32, new_inner_height: u32);
}

pub struct WindowingSystem {
    pub(crate) winit_window: winit::window::Window,
    pub(crate) winit_event_loop_proxy: winit::event_loop::EventLoopProxy<Request>,
    pub(crate) force_quit: bool,
    // TODO: Refactor into better event system
    pub(crate) resize_listeners: Vec<Rc<RefCell<dyn ResizeListener>>>,
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

    /// Adds a resize listener (see trait ``ResizeListener`) that will be notified whenever
    /// a resize event occurs.
    pub(crate) fn add_resize_listener(&mut self, resize_listener: Rc<RefCell<dyn ResizeListener>>) {
        self.resize_listeners.push(resize_listener);
    }

    /// Notifies all resize listeners (registered with `add_resize_listener()`, of a resize)
    pub(crate) fn notify_resize(&self, new_inner_width: u32, new_inner_height: u32) {
        for x in &self.resize_listeners {
            x.borrow_mut()
                .resize_event(new_inner_width, new_inner_height);
        }
    }
}

/// Represents a request to the windowing system in the form of an UserEvent.
#[derive(Debug)]
pub(crate) enum Request {
    Quit,
}
