use napi_derive::napi;

#[napi(object)]
pub struct MacWindowBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[napi(object)]
pub struct MacWindow {
    /// Stable per-session window id. Matches the trailing number in
    /// Electron's `desktopCapturer` source id `window:<windowId>:<displayIndex>`.
    pub window_id: u32,
    /// Owning process bundle/executable name, e.g. "Safari", "Code", "Workspace".
    pub owner_name: String,
    pub owner_pid: i32,
    /// Window title. May be null when the app does not expose one or when the
    /// caller lacks Screen Recording permission.
    pub title: Option<String>,
    pub bounds: MacWindowBounds,
    /// Window server layer. 0 == normal app window. Non-zero usually means
    /// menu bar, dock, status bar, or other system chrome.
    pub layer: i32,
    /// Whether the window is currently visible on a display.
    pub on_screen: bool,
}

#[napi(object)]
pub struct ListWindowsOptions {
    /// When true (default), only return windows currently visible on a display.
    pub on_screen_only: Option<bool>,
    /// When true (default), exclude desktop wallpaper / icon elements.
    pub exclude_desktop_elements: Option<bool>,
    /// When true, also include windows on layers != 0 (menu bar, dock, etc.).
    /// Default: false - most callers only care about normal app windows.
    pub include_system_chrome: Option<bool>,
}
