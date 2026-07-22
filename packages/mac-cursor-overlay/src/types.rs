use napi_derive::napi;

#[napi(object)]
pub struct OverlayBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[napi(object)]
pub struct OverlayPoint {
    pub x: f64,
    pub y: f64,
}

#[napi(string_enum)]
pub enum OverlayLevel {
    Normal,
    Floating,
    TornOffMenu,
    ModalPanel,
    MainMenu,
    Status,
    PopUpMenu,
    ScreenSaver,
}

#[napi(object)]
pub struct OverlayState {
    pub has_window: bool,
    pub window_visible: bool,
    pub cursor_visible: bool,
    pub debug_backdrop_visible: bool,
    pub level: String,
    pub bounds: OverlayBounds,
    pub cursor_position: OverlayPoint,
}
