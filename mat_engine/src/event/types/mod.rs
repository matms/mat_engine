//! Event types
#[derive(Debug, Copy, Clone)]
pub enum EventType {
    WindowResizeEvent,
    PreUpdateEvent,
    PostUpdateEvent,
    PreRenderEvent,
    PostRenderEvent,
}
