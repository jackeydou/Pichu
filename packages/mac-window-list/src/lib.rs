#![cfg(target_os = "macos")]
#![deny(clippy::all)]

mod ffi;
mod types;
mod window_list;

pub use types::{ListWindowsOptions, MacWindow, MacWindowBounds};
pub use window_list::list_windows;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_call() {
        // This won't necessarily return anything in CI without a session, but it
        // must not panic and should at least give us back an empty Vec.
        let _ = list_windows(None);
    }
}
