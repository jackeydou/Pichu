use crate::ffi::{NativePoint, NativeRect};
use crate::types::{OverlayBounds, OverlayLevel, OverlayPoint};
use napi::bindgen_prelude::*;

pub(crate) fn level_to_native(level: &OverlayLevel) -> i32 {
    match level {
        OverlayLevel::Normal => 0,
        OverlayLevel::Floating => 1,
        OverlayLevel::TornOffMenu => 2,
        OverlayLevel::ModalPanel => 3,
        OverlayLevel::MainMenu => 4,
        OverlayLevel::Status => 5,
        OverlayLevel::PopUpMenu => 6,
        OverlayLevel::ScreenSaver => 7,
    }
}

pub(crate) fn level_from_native(level: i32) -> String {
    match level {
        0 => "normal",
        1 => "floating",
        2 => "torn-off-menu",
        3 => "modal-panel",
        4 => "main-menu",
        5 => "status",
        6 => "pop-up-menu",
        7 => "screen-saver",
        _ => "floating",
    }
    .to_string()
}

pub(crate) fn to_native_rect(bounds: OverlayBounds) -> Result<NativeRect> {
    if ![bounds.x, bounds.y, bounds.width, bounds.height]
        .into_iter()
        .all(|value| value.is_finite())
    {
        return Err(Error::new(
            Status::InvalidArg,
            format!(
                "Overlay bounds must be finite numbers (got x={}, y={}, width={}, height={}).",
                bounds.x, bounds.y, bounds.width, bounds.height
            ),
        ));
    }
    Ok(NativeRect {
        x: bounds.x,
        y: bounds.y,
        width: bounds.width.max(1.0),
        height: bounds.height.max(1.0),
    })
}

pub(crate) fn to_native_point(point: OverlayPoint) -> Result<NativePoint> {
    if ![point.x, point.y]
        .into_iter()
        .all(|value| value.is_finite())
    {
        return Err(Error::new(
            Status::InvalidArg,
            format!(
                "Overlay points must be finite numbers (got x={}, y={}).",
                point.x, point.y
            ),
        ));
    }
    Ok(NativePoint {
        x: point.x,
        y: point.y,
    })
}
