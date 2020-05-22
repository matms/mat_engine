//! Event stuff
//! Pertains to engine events: not to be used for user events.

pub mod events;
pub mod types;

#[derive(Debug, Copy, Clone)]
pub enum Event {
    /// Called once the window is resized.
    WindowResizeEvent(events::WindowResizeEvent),
    /// Called before `Application::update()`
    PreUpdateEvent,
    /// Called right after `Application::update()`
    PostUpdateEvent,
    /// Called before `Application::render()`
    PreRenderEvent,
    /// Called just after `Application::render()`
    PostRenderEvent,
}

impl Event {
    fn event_type(&self) -> types::EventType {
        match self {
            Event::WindowResizeEvent(_) => types::EventType::WindowResizeEvent,
            Event::PreUpdateEvent => types::EventType::PreUpdateEvent,
            Event::PostUpdateEvent => types::EventType::PostUpdateEvent,
            Event::PreRenderEvent => types::EventType::PreRenderEvent,
            Event::PostRenderEvent => types::EventType::PostRenderEvent,
        }
    }
}

/// Internal engine systems only
pub trait EventReceiver {
    fn receives_event_type(evt_type: types::EventType) -> bool;

    /// Will be called iff `receives_event_type()` returns true for the event's type
    fn receive_event(ctx: &mut crate::EngineContext, evt: Event);
}

pub trait ApplicationEventReceiver {
    fn receives_event_type(evt_type: types::EventType) -> bool {
        false
    }

    /// Will be called iff `receives_event_type()` returns true for the event's type
    fn receive_event(&mut self, ctx: &mut crate::EngineContext, evt: Event) {}
}

/// Calls receive_event iff `receives_event_type` is true.
pub(super) fn inform_receiver<T: EventReceiver>(ctx: &mut crate::EngineContext, evt: Event) {
    if T::receives_event_type(evt.event_type()) {
        T::receive_event(ctx, evt)
    }
}

pub(super) fn inform_application<T: super::application::Application>(
    app: &mut T,
    ctx: &mut crate::EngineContext,
    evt: Event,
) {
}

/// FIFO Queue of engine events. Not to be used for user events
pub(crate) struct EventQueue {
    queue: std::collections::VecDeque<Event>,
}

impl EventQueue {
    pub(crate) fn new() -> Self {
        Self {
            queue: std::collections::VecDeque::new(),
        }
    }

    /// Adds an event to the end of the queue.
    pub(crate) fn push_event(&mut self, evt: Event) {
        self.queue.push_back(evt);
    }

    /// Takes the first event, removes it from the queue, and returns it.
    /// Returns None if there are no events in the queue.
    pub(crate) fn retrieve_event(&mut self) -> Option<Event> {
        self.queue.pop_front()
    }
}

/// Dummy event receiver for debugging purposes
pub(super) struct DebugEventReceiver {}

impl EventReceiver for DebugEventReceiver {
    fn receives_event_type(evt_type: types::EventType) -> bool {
        true
    }
    fn receive_event(ctx: &mut crate::EngineContext, evt: Event) {
        log::trace!("Event: {:?}", evt);
    }
}
