use crate::utils::unwrap_ref;

/// Returns whether the cursor is inside the screen.
/// If state is unknown, returns false
pub fn is_cursor_inside_screen(ctx: &crate::context::EngineContext) -> bool {
    let (state, _) = get_cursor_info(ctx);

    match state {
        CursorState::Unknown => false,
        CursorState::OutsideScreen => false,
        CursorState::InsideScreen => true,
    }
}

/// Returns an option, with the None option if the cursor is outside the screen (or unknown).
pub fn get_cursor_position(ctx: &crate::context::EngineContext) -> Option<CursorPosition> {
    let (_, pos) = get_cursor_info(ctx);
    pos
}

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
