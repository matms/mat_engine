use crate::utils::unwrap_mut;

/// Sends a quit request to winit's event loop. This will, (possibly after a delay, as
/// queued events will still be processed), cause the loop to exit. `Application::close()`
/// will be automatically called, there is no need for you to call it.
///
/// This is a wrapper method.
pub fn queue_quit(ctx: &mut crate::EngineContext) {
    unwrap_mut(&mut ctx.windowing_system).queue_quit()
}

/// Forces winit's event loop to quit, ignoring all outstanding winit events.
/// `Application::close()` will be automatically called, there is no need for
/// you to call it.
///
/// This is a wrapper method.
pub fn force_quit(ctx: &mut crate::EngineContext) {
    unwrap_mut(&mut ctx.windowing_system).force_quit()
}

pub(crate) fn make_winit_event_loop() -> winit::event_loop::EventLoop<Request> {
    winit::event_loop::EventLoop::<Request>::with_user_event()
}

pub(crate) fn make_winit_event_loop_proxy(
    winit_event_loop: &winit::event_loop::EventLoop<Request>,
) -> winit::event_loop::EventLoopProxy<Request> {
    winit_event_loop.create_proxy()
}

pub(crate) fn make_default_winit_window(
    winit_event_loop: &winit::event_loop::EventLoop<Request>,
) -> winit::window::Window {
    winit::window::WindowBuilder::new()
        .with_title("Sample Application")
        .build(&winit_event_loop)
        .expect("Could not obtain winit window")
}

pub struct WindowingSystem {
    pub(crate) winit_window: winit::window::Window,
    pub(crate) winit_event_loop_proxy: winit::event_loop::EventLoopProxy<Request>,
    pub(crate) force_quit: bool,
}

impl WindowingSystem {
    pub(crate) fn new(
        winit_window: winit::window::Window,
        winit_event_loop_proxy: winit::event_loop::EventLoopProxy<Request>,
    ) -> Self {
        Self {
            winit_window,
            winit_event_loop_proxy,
            force_quit: false,
        }
    }

    pub(crate) fn get_window_ref(&self) -> &winit::window::Window {
        &self.winit_window
    }

    /// See wrapper method `windowing::queue_quit()`.
    fn queue_quit(&mut self) {
        self.winit_event_loop_proxy
            .send_event(Request::Quit)
            .expect("Couldn't send event to winit event loop.");
    }

    /// See wrapper method `windowing::force_quit()`.
    fn force_quit(&mut self) {
        self.force_quit = true;
    }
}

/// TODO: Refactor into better, more general event system.
///
/// Represents a request to the windowing system* in the form of an UserEvent.
/// Used to interface with winit's event loop.
///
/// *(Does it? Or is it better described as a request to winit's event loop _from_ the windowing system?)
/// TODO Investigate.
#[derive(Debug)]
pub enum Request {
    Quit,
}
