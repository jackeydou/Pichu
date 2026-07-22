#![cfg(target_os = "macos")]
#![deny(clippy::all)]

mod api;
mod conversions;
mod ffi;
mod types;

pub use api::{
    dispose_overlay, flash_click, get_overlay_state, hide_overlay, jump_cursor,
    set_attached_window_id, set_cursor_pressed, set_cursor_visible, set_debug_backdrop,
    set_overlay_bounds, set_overlay_level, show_overlay,
};
pub use types::{OverlayBounds, OverlayLevel, OverlayPoint, OverlayState};
