use crate::platform::*;
use napi::bindgen_prelude::*;
use std::thread;
use std::time::Duration;

// ---------- Internal posting helpers ----------

pub(crate) fn post_mouse_event(
    source: &OwnedSource,
    event_type: CGEventType,
    pos: CGPoint,
    button: CGMouseButton,
    click_state: Option<i64>,
    flags: CGEventFlags,
) -> Result<()> {
    let raw = unsafe { CGEventCreateMouseEvent(source.0, event_type, pos, button) };
    if raw.is_null() {
        return Err(Error::new(
            Status::GenericFailure,
            "CGEventCreateMouseEvent returned null",
        ));
    }
    let event = OwnedEvent(raw);
    if let Some(state) = click_state {
        unsafe { CGEventSetIntegerValueField(event.0, KCG_MOUSE_EVENT_CLICK_STATE, state) };
    }
    if flags != 0 {
        unsafe { CGEventSetFlags(event.0, flags) };
    }
    unsafe { CGEventPost(KCG_HID_EVENT_TAP, event.0) };
    Ok(())
}

pub(crate) fn sleep_ms(ms: u32) {
    if ms > 0 {
        thread::sleep(Duration::from_millis(ms as u64));
    }
}

pub(crate) fn ns_event_type_for_mouse(event_type: CGEventType) -> Result<NSUInteger> {
    match event_type {
        KCG_EVENT_LEFT_MOUSE_DOWN
        | KCG_EVENT_LEFT_MOUSE_UP
        | KCG_EVENT_RIGHT_MOUSE_DOWN
        | KCG_EVENT_RIGHT_MOUSE_UP
        | KCG_EVENT_MOUSE_MOVED
        | KCG_EVENT_LEFT_MOUSE_DRAGGED
        | KCG_EVENT_RIGHT_MOUSE_DRAGGED
        | KCG_EVENT_OTHER_MOUSE_DOWN
        | KCG_EVENT_OTHER_MOUSE_UP
        | KCG_EVENT_OTHER_MOUSE_DRAGGED => Ok(event_type as NSUInteger),
        _ => Err(Error::new(
            Status::InvalidArg,
            format!(
                "Unsupported mouse event type for NSEvent synthesis: {}",
                event_type
            ),
        )),
    }
}

pub(crate) fn pressure_for_mouse(event_type: CGEventType) -> f32 {
    match event_type {
        KCG_EVENT_LEFT_MOUSE_UP | KCG_EVENT_RIGHT_MOUSE_UP | KCG_EVENT_OTHER_MOUSE_UP => 0.0,
        _ => 1.0,
    }
}

/// Build a mouse event through AppKit (`NSEvent.mouseEvent(...).CGEvent`) so
/// the event shape matches the reverse-engineered background-click path more
/// closely than a bare `CGEventCreateMouseEvent`.
pub(crate) fn create_appkit_mouse_event(
    event_type: CGEventType,
    screen: CGPoint,
    window_id: u32,
    flags: CGEventFlags,
    event_number: i64,
    click_count: i64,
) -> Result<OwnedEvent> {
    let ns_event_type = ns_event_type_for_mouse(event_type)?;
    let pool = unsafe { objc_autoreleasePoolPush() };
    let result = (|| -> Result<OwnedEvent> {
        let ns_event_class = class(b"NSEvent\0");
        if ns_event_class.is_null() {
            return Err(Error::new(
                Status::GenericFailure,
                "Could not resolve NSEvent class",
            ));
        }
        let ns_event = unsafe {
            msg_nsevent_mouse_event()(
        ns_event_class as ObjcId,
        sel(
          b"mouseEventWithType:location:modifierFlags:timestamp:windowNumber:context:eventNumber:clickCount:pressure:\0",
        ),
        ns_event_type,
        screen,
        flags as NSUInteger,
        system_uptime(),
        window_id as NSInteger,
        std::ptr::null_mut(),
        event_number as NSInteger,
        click_count as NSInteger,
        pressure_for_mouse(event_type),
      )
        };
        if ns_event.is_null() {
            return Err(Error::new(
                Status::GenericFailure,
                "NSEvent.mouseEventWithType returned null",
            ));
        }
        let cg_event = unsafe { msg_cg_event()(ns_event, sel(b"CGEvent\0")) };
        if cg_event.is_null() {
            return Err(Error::new(
                Status::GenericFailure,
                "NSEvent.CGEvent returned null",
            ));
        }
        // `-[NSEvent CGEvent]` follows getter semantics, so retain before the
        // autorelease pool drains and transfer ownership to `OwnedEvent`.
        unsafe {
            CFRetain(cg_event as CFTypeRef);
        }
        Ok(OwnedEvent(cg_event))
    })();
    unsafe {
        objc_autoreleasePoolPop(pool);
    }
    result
}
