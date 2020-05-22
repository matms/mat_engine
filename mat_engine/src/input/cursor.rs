use crate::utils::unwrap_ref;

pub fn get_cursor_info(
    ctx: &crate::context::EngineContext,
) -> (CursorState, Option<CursorPosition>) {
    let cs = unwrap_ref(&ctx.input_system).cursor_state;

    if let CursorState::InsideScreen = cs {
        (cs, Some(unwrap_ref(&ctx.input_system).cursor_pos))
    } else {
        (cs, None)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum CursorState {
    Unknown,
    OutsideScreen,
    InsideScreen,
}

#[derive(Debug, Copy, Clone)]
pub struct CursorPosition {
    pub x: f64,
    pub y: f64,
}
