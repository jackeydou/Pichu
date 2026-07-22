use napi::bindgen_prelude::*;
use std::os::raw::{c_char, c_void};
use std::sync::atomic::{AtomicI64, AtomicPtr, Ordering};
use std::sync::OnceLock;

// ---------- CoreGraphics FFI ----------

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct CGPoint {
    pub(crate) x: f64,
    pub(crate) y: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct CGSize {
    pub(crate) width: f64,
    pub(crate) height: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct CGRect {
    pub(crate) origin: CGPoint,
    pub(crate) size: CGSize,
}

pub(crate) type CFTypeRef = *const c_void;
pub(crate) type CFBooleanRef = *const c_void;
pub(crate) type CFURLRef = *const c_void;
pub(crate) type CFDictionaryRef = *const c_void;
pub(crate) type CGEventRef = *mut c_void;
pub(crate) type CGEventSourceRef = *mut c_void;
pub(crate) type CGImageRef = *mut c_void;
pub(crate) type CGImageDestinationRef = *mut c_void;
pub(crate) type NSInteger = isize;
pub(crate) type NSUInteger = usize;

pub(crate) type CGEventType = u32;
pub(crate) type CGMouseButton = u32;
pub(crate) type CGEventFlags = u64;
pub(crate) type CGEventField = u32;
pub(crate) type CGEventTapLocation = u32;
pub(crate) type CGEventSourceStateID = i32;
pub(crate) type CGKeyCode = u16;
pub(crate) type CGWindowID = u32;
pub(crate) type CGWindowListOption = u32;
pub(crate) type CGWindowImageOption = u32;
pub(crate) type CGDirectDisplayID = u32;

pub(crate) const KCG_EVENT_LEFT_MOUSE_DOWN: CGEventType = 1;
pub(crate) const KCG_EVENT_LEFT_MOUSE_UP: CGEventType = 2;
pub(crate) const KCG_EVENT_RIGHT_MOUSE_DOWN: CGEventType = 3;
pub(crate) const KCG_EVENT_RIGHT_MOUSE_UP: CGEventType = 4;
pub(crate) const KCG_EVENT_MOUSE_MOVED: CGEventType = 5;
pub(crate) const KCG_EVENT_LEFT_MOUSE_DRAGGED: CGEventType = 6;
pub(crate) const KCG_EVENT_RIGHT_MOUSE_DRAGGED: CGEventType = 7;
pub(crate) const KCG_EVENT_OTHER_MOUSE_DOWN: CGEventType = 25;
pub(crate) const KCG_EVENT_OTHER_MOUSE_UP: CGEventType = 26;
pub(crate) const KCG_EVENT_OTHER_MOUSE_DRAGGED: CGEventType = 27;

pub(crate) const KCG_MOUSE_BUTTON_LEFT: CGMouseButton = 0;
pub(crate) const KCG_MOUSE_BUTTON_RIGHT: CGMouseButton = 1;
pub(crate) const KCG_MOUSE_BUTTON_CENTER: CGMouseButton = 2;

pub(crate) const KCG_EVENT_FLAG_MASK_SHIFT: CGEventFlags = 0x00020000;
pub(crate) const KCG_EVENT_FLAG_MASK_CONTROL: CGEventFlags = 0x00040000;
pub(crate) const KCG_EVENT_FLAG_MASK_ALTERNATE: CGEventFlags = 0x00080000;
pub(crate) const KCG_EVENT_FLAG_MASK_COMMAND: CGEventFlags = 0x00100000;
pub(crate) const KCG_EVENT_FLAG_MASK_FN: CGEventFlags = 0x00800000;

// Public CGEventField constants (CoreGraphics/CGEventTypes.h).
pub(crate) const KCG_MOUSE_EVENT_CLICK_STATE: CGEventField = 1;
pub(crate) const KCG_MOUSE_EVENT_BUTTON_NUMBER: CGEventField = 3;
/// `kCGMouseEventSubtype` — value 3 is the non-public "synthesized window-targeted" subtype.
pub(crate) const KCG_MOUSE_EVENT_SUBTYPE: CGEventField = 7;
pub(crate) const KCG_MOUSE_EVENT_WINDOW_UNDER_MOUSE_POINTER: CGEventField = 91;
pub(crate) const KCG_MOUSE_EVENT_WINDOW_UNDER_MOUSE_POINTER_THAT_CAN_HANDLE_THIS_EVENT:
    CGEventField = 92;

pub(crate) const KCG_HID_EVENT_TAP: CGEventTapLocation = 0;

pub(crate) const KCG_EVENT_SOURCE_STATE_HID_SYSTEM_STATE: CGEventSourceStateID = 1;
pub(crate) const KCG_WINDOW_LIST_OPTION_INCLUDING_WINDOW: CGWindowListOption = 1 << 3;
pub(crate) const KCG_WINDOW_IMAGE_BOUNDS_IGNORE_FRAMING: CGWindowImageOption = 1 << 0;
pub(crate) const KCG_WINDOW_IMAGE_BEST_RESOLUTION: CGWindowImageOption = 1 << 3;

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    pub(crate) static CGRectNull: CGRect;
    pub(crate) fn CGEventSourceCreate(state_id: CGEventSourceStateID) -> CGEventSourceRef;
    pub(crate) fn CGEventCreateMouseEvent(
        source: CGEventSourceRef,
        mouse_type: CGEventType,
        mouse_cursor_position: CGPoint,
        mouse_button: CGMouseButton,
    ) -> CGEventRef;
    pub(crate) fn CGEventCreateKeyboardEvent(
        source: CGEventSourceRef,
        virtual_key: CGKeyCode,
        key_down: bool,
    ) -> CGEventRef;
    pub(crate) fn CGEventKeyboardSetUnicodeString(
        event: CGEventRef,
        string_length: usize,
        unicode_string: *const u16,
    );
    pub(crate) fn CGEventGetLocation(event: CGEventRef) -> CGPoint;
    pub(crate) fn CGEventSetIntegerValueField(event: CGEventRef, field: CGEventField, value: i64);
    pub(crate) fn CGEventSetFlags(event: CGEventRef, flags: CGEventFlags);
    pub(crate) fn CGEventSetLocation(event: CGEventRef, location: CGPoint);
    pub(crate) fn CGEventPost(tap: CGEventTapLocation, event: CGEventRef);
    /// Posts the event directly to the target process, bypassing the normal
    /// system-wide event tap chain. This is the foundation of "background
    /// click" — events are delivered to a non-frontmost app without activating it.
    pub(crate) fn CGEventPostToPid(pid: i32, event: CGEventRef);
    pub(crate) fn CGWindowListCreateImage(
        screen_bounds: CGRect,
        list_option: CGWindowListOption,
        window_id: CGWindowID,
        image_option: CGWindowImageOption,
    ) -> CGImageRef;
    pub(crate) fn CGMainDisplayID() -> CGDirectDisplayID;
    pub(crate) fn CGDisplayCreateImage(display_id: CGDirectDisplayID) -> CGImageRef;
}

// dlsym for the private `CGEventSetWindowLocation`. Documented to exist on
// every macOS version with WindowServer event delivery (≥10.4). Resolving via
// dlsym keeps us out of `__DATA,__la_symbol_ptr` so we don't show up in
// `nm -u` and don't break older SDKs.
pub(crate) type CGEventSetWindowLocationFn =
    unsafe extern "C" fn(event: CGEventRef, location: CGPoint);
pub(crate) const RTLD_DEFAULT: *mut c_void = -2isize as *mut c_void;

#[link(name = "c")]
unsafe extern "C" {
    pub(crate) fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
}

pub(crate) fn cg_event_set_window_location() -> Option<CGEventSetWindowLocationFn> {
    static FN: OnceLock<Option<usize>> = OnceLock::new();
    let resolved = FN.get_or_init(|| unsafe {
        let name = b"CGEventSetWindowLocation\0".as_ptr() as *const c_char;
        let p = dlsym(RTLD_DEFAULT, name);
        if p.is_null() {
            None
        } else {
            Some(p as usize)
        }
    });
    resolved.map(|addr| unsafe { std::mem::transmute::<usize, CGEventSetWindowLocationFn>(addr) })
}

// ---------- Objective-C runtime (NSWorkspace / NSRunningApplication) ----------
//
// We need two answers from AppKit at runtime:
//   1. Which pid is currently the foreground app? (`NSWorkspace.shared.frontmostApplication.processIdentifier`)
//   2. Is a given pid the foreground app? (`NSRunningApplication(pid:).isActive`)
//
// Pulling in `objc` / `objc2` / `cocoa` crates is overkill — we only need
// three selectors. We talk to libobjc directly.

pub(crate) type ObjcId = *mut c_void;
pub(crate) type ObjcSel = *mut c_void;
pub(crate) type ObjcClass = *mut c_void;

#[link(name = "objc")]
unsafe extern "C" {
    pub(crate) fn objc_getClass(name: *const c_char) -> ObjcClass;
    pub(crate) fn sel_registerName(name: *const c_char) -> ObjcSel;
    pub(crate) fn objc_autoreleasePoolPush() -> *mut c_void;
    pub(crate) fn objc_autoreleasePoolPop(pool: *mut c_void);
    // objc_msgSend is variadic; we declare it as opaque and transmute to the
    // exact signature for each call site.
    pub(crate) fn objc_msgSend();
}

#[link(name = "AppKit", kind = "framework")]
unsafe extern "C" {}

type MsgSendReturnId = unsafe extern "C" fn(ObjcId, ObjcSel) -> ObjcId;
type MsgSendReturnPid = unsafe extern "C" fn(ObjcId, ObjcSel) -> i32;
type MsgSendReturnBool = unsafe extern "C" fn(ObjcId, ObjcSel) -> bool;
type MsgSendReturnF64 = unsafe extern "C" fn(ObjcId, ObjcSel) -> f64;
type MsgSendIdArgPid = unsafe extern "C" fn(ObjcId, ObjcSel, i32) -> ObjcId;
type MsgSendBoolArgNSUInteger = unsafe extern "C" fn(ObjcId, ObjcSel, NSUInteger) -> bool;
type MsgSendNSEventMouseEvent = unsafe extern "C" fn(
    ObjcId,
    ObjcSel,
    NSUInteger,
    CGPoint,
    NSUInteger,
    f64,
    NSInteger,
    ObjcId,
    NSInteger,
    NSInteger,
    f32,
) -> ObjcId;
type MsgSendReturnCGEvent = unsafe extern "C" fn(ObjcId, ObjcSel) -> CGEventRef;

pub(crate) fn msg_id() -> MsgSendReturnId {
    unsafe { std::mem::transmute(objc_msgSend as *const ()) }
}
pub(crate) fn msg_pid() -> MsgSendReturnPid {
    unsafe { std::mem::transmute(objc_msgSend as *const ()) }
}
pub(crate) fn msg_bool() -> MsgSendReturnBool {
    unsafe { std::mem::transmute(objc_msgSend as *const ()) }
}
pub(crate) fn msg_f64() -> MsgSendReturnF64 {
    unsafe { std::mem::transmute(objc_msgSend as *const ()) }
}
pub(crate) fn msg_id_arg_pid() -> MsgSendIdArgPid {
    unsafe { std::mem::transmute(objc_msgSend as *const ()) }
}
pub(crate) fn msg_bool_arg_nsuint() -> MsgSendBoolArgNSUInteger {
    unsafe { std::mem::transmute(objc_msgSend as *const ()) }
}
pub(crate) fn msg_nsevent_mouse_event() -> MsgSendNSEventMouseEvent {
    unsafe { std::mem::transmute(objc_msgSend as *const ()) }
}
pub(crate) fn msg_cg_event() -> MsgSendReturnCGEvent {
    unsafe { std::mem::transmute(objc_msgSend as *const ()) }
}

pub(crate) fn sel(name: &[u8]) -> ObjcSel {
    // Caller passes a NUL-terminated byte slice (b"foo\0").
    debug_assert_eq!(
        name.last(),
        Some(&0u8),
        "selector name must be NUL-terminated"
    );
    unsafe { sel_registerName(name.as_ptr() as *const c_char) }
}

pub(crate) fn class(name: &[u8]) -> ObjcClass {
    debug_assert_eq!(name.last(), Some(&0u8), "class name must be NUL-terminated");
    unsafe { objc_getClass(name.as_ptr() as *const c_char) }
}

/// Returns the pid of the current frontmost (foreground) macOS app, or None
/// if AppKit hasn't initialized a workspace (very rare).
pub(crate) fn frontmost_pid() -> Option<i32> {
    let workspace_class = class(b"NSWorkspace\0");
    if workspace_class.is_null() {
        return None;
    }
    unsafe {
        let workspace = msg_id()(workspace_class, sel(b"sharedWorkspace\0"));
        if workspace.is_null() {
            return None;
        }
        let app = msg_id()(workspace, sel(b"frontmostApplication\0"));
        if app.is_null() {
            return None;
        }
        let pid = msg_pid()(app, sel(b"processIdentifier\0"));
        Some(pid)
    }
}

/// Returns true when the given pid is currently the foreground app.
/// Implemented via `+[NSRunningApplication runningApplicationWithProcessIdentifier:]`
/// then `-[NSRunningApplication isActive]` so it stays in sync with how the
/// bgclick-rev skill recommends gating the WindowServer bypass flag.
pub(crate) fn pid_is_active(pid: i32) -> bool {
    let cls = class(b"NSRunningApplication\0");
    if cls.is_null() {
        return false;
    }
    unsafe {
        let app = msg_id_arg_pid()(
            cls as ObjcId,
            sel(b"runningApplicationWithProcessIdentifier:\0"),
            pid,
        );
        if app.is_null() {
            return false;
        }
        msg_bool()(app, sel(b"isActive\0"))
    }
}

pub(crate) fn activate_pid(pid: i32) -> bool {
    let cls = class(b"NSRunningApplication\0");
    if cls.is_null() {
        return false;
    }
    unsafe {
        let app = msg_id_arg_pid()(
            cls as ObjcId,
            sel(b"runningApplicationWithProcessIdentifier:\0"),
            pid,
        );
        if app.is_null() {
            return false;
        }
        // NSApplicationActivateIgnoringOtherApps = 1 << 1.
        msg_bool_arg_nsuint()(app, sel(b"activateWithOptions:\0"), 2)
    }
}

pub(crate) fn system_uptime() -> f64 {
    let cls = class(b"NSProcessInfo\0");
    if cls.is_null() {
        return 0.0;
    }
    unsafe {
        let process = msg_id()(cls as ObjcId, sel(b"processInfo\0"));
        if process.is_null() {
            return 0.0;
        }
        msg_f64()(process, sel(b"systemUptime\0"))
    }
}

// Process-global monotonically increasing event number. Matches the pattern
// the upstream skill describes — the target uses a global counter or
// `systemUptime * 1e6 & 0x7fff_ffff`. Either works because AppKit only checks
// uniqueness within a small recent window.
pub(crate) static EVENT_NUMBER_COUNTER: AtomicI64 = AtomicI64::new(1);
pub(crate) fn next_event_number() -> i64 {
    EVENT_NUMBER_COUNTER.fetch_add(1, Ordering::Relaxed)
}

// Cache the resolved CGEventSetWindowLocation pointer for a faster hot path.
// We hold the function pointer as a usize inside an AtomicPtr-wrapped slot
// because raw fn pointers are not Send.
pub(crate) static SET_WINDOW_LOCATION_CACHED: AtomicPtr<c_void> =
    AtomicPtr::new(std::ptr::null_mut());
pub(crate) fn cached_set_window_location() -> Option<CGEventSetWindowLocationFn> {
    let p = SET_WINDOW_LOCATION_CACHED.load(Ordering::Relaxed);
    if p.is_null() {
        let resolved = cg_event_set_window_location()?;
        SET_WINDOW_LOCATION_CACHED.store(resolved as *mut c_void, Ordering::Relaxed);
        Some(resolved)
    } else {
        Some(unsafe { std::mem::transmute::<*mut c_void, CGEventSetWindowLocationFn>(p) })
    }
}

pub(crate) type CFStringRef = *const c_void;
pub(crate) type CFArrayRef = *const c_void;
pub(crate) type CFAllocatorRef = *const c_void;
pub(crate) type CFIndex = isize;
pub(crate) type CFTypeID = usize;
pub(crate) type CFURLPathStyle = CFIndex;

pub(crate) const KCF_STRING_ENCODING_UTF8: u32 = 0x0800_0100;
pub(crate) const KCF_URL_POSIX_PATH_STYLE: CFURLPathStyle = 0;

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    pub(crate) static kCFBooleanTrue: CFBooleanRef;
    pub(crate) fn CFGetTypeID(cf: CFTypeRef) -> CFTypeID;
    pub(crate) fn CFArrayGetTypeID() -> CFTypeID;
    pub(crate) fn CFStringGetTypeID() -> CFTypeID;
    pub(crate) fn CFRetain(cf: CFTypeRef) -> CFTypeRef;
    pub(crate) fn CFRelease(cf: CFTypeRef);
    pub(crate) fn CFEqual(cf1: CFTypeRef, cf2: CFTypeRef) -> bool;
    pub(crate) fn CFCopyDescription(cf: CFTypeRef) -> CFStringRef;
    pub(crate) fn CFStringGetLength(the_string: CFStringRef) -> CFIndex;
    pub(crate) fn CFStringGetMaximumSizeForEncoding(length: CFIndex, encoding: u32) -> CFIndex;
    pub(crate) fn CFStringGetCString(
        the_string: CFStringRef,
        buffer: *mut c_char,
        buffer_size: CFIndex,
        encoding: u32,
    ) -> u8;
    pub(crate) fn CFBooleanGetTypeID() -> CFTypeID;
    pub(crate) fn CFNumberGetTypeID() -> CFTypeID;
    pub(crate) fn CFStringCreateWithCString(
        allocator: CFAllocatorRef,
        cstr: *const c_char,
        encoding: u32,
    ) -> CFStringRef;
    pub(crate) fn CFURLCreateWithFileSystemPath(
        allocator: CFAllocatorRef,
        file_path: CFStringRef,
        path_style: CFURLPathStyle,
        is_directory: u8,
    ) -> CFURLRef;
    pub(crate) fn CFArrayGetCount(array: CFArrayRef) -> CFIndex;
    pub(crate) fn CFArrayGetValueAtIndex(array: CFArrayRef, idx: CFIndex) -> *const c_void;
}

#[link(name = "ImageIO", kind = "framework")]
unsafe extern "C" {
    pub(crate) fn CGImageDestinationCreateWithURL(
        url: CFURLRef,
        image_type: CFStringRef,
        count: usize,
        options: CFDictionaryRef,
    ) -> CGImageDestinationRef;
    pub(crate) fn CGImageDestinationAddImage(
        destination: CGImageDestinationRef,
        image: CGImageRef,
        properties: CFDictionaryRef,
    );
    pub(crate) fn CGImageDestinationFinalize(destination: CGImageDestinationRef) -> bool;
}

// AXUIElementRef is a CFType (CFRelease-able). Treat it as opaque.
pub(crate) type AXUIElementRef = *mut c_void;
pub(crate) type AXValueRef = *const c_void;
pub(crate) type AXValueType = u32;
pub(crate) type AXError = i32;
pub(crate) const KAX_ERROR_SUCCESS: AXError = 0;
pub(crate) const KAX_VALUE_CGPOINT_TYPE: AXValueType = 1;
pub(crate) const KAX_VALUE_CGSIZE_TYPE: AXValueType = 2;

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    /// `AXIsProcessTrusted()` reports whether the current process appears in
    /// System Settings → Privacy & Security → Accessibility with the toggle on.
    /// Without it, posted CGEvents are silently dropped by the WindowServer.
    pub(crate) fn AXIsProcessTrusted() -> bool;

    /// Get an AXUIElement representing the application with the given pid.
    /// Returns a +1 retained AXUIElementRef — must be CFReleased.
    pub(crate) fn AXUIElementCreateApplication(pid: i32) -> AXUIElementRef;

    /// Read an attribute (e.g. AXWindows). On success, *value is a +1 retained
    /// CFTypeRef — caller CFReleases it.
    pub(crate) fn AXUIElementCopyAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: *mut CFTypeRef,
    ) -> AXError;
    pub(crate) fn AXUIElementSetAttributeValue(
        element: AXUIElementRef,
        attribute: CFStringRef,
        value: CFTypeRef,
    ) -> AXError;
    pub(crate) fn AXUIElementCopyAttributeNames(
        element: AXUIElementRef,
        names: *mut CFArrayRef,
    ) -> AXError;
    pub(crate) fn AXUIElementCopyActionNames(
        element: AXUIElementRef,
        names: *mut CFArrayRef,
    ) -> AXError;
    pub(crate) fn AXUIElementIsAttributeSettable(
        element: AXUIElementRef,
        attribute: CFStringRef,
        settable: *mut bool,
    ) -> AXError;

    /// Perform an AX action on the element (e.g. "AXRaise").
    pub(crate) fn AXUIElementPerformAction(element: AXUIElementRef, action: CFStringRef)
        -> AXError;
    pub(crate) fn AXUIElementGetPid(element: AXUIElementRef, pid: *mut i32) -> AXError;
    pub(crate) fn AXUIElementGetTypeID() -> CFTypeID;
    pub(crate) fn AXValueGetType(value: AXValueRef) -> AXValueType;
    pub(crate) fn AXValueGetValue(
        value: AXValueRef,
        the_type: AXValueType,
        value_ptr: *mut c_void,
    ) -> bool;
}

// Private but stable since 10.10. Returns the CGWindowID for an AX window
// element. Used by Hammerspoon, Karabiner-Elements, and many other automation
// tools — well-known and unlikely to disappear.
#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    pub(crate) fn _AXUIElementGetWindow(element: AXUIElementRef, window_id: *mut u32) -> AXError;
}

// ---------- RAII helpers ----------

pub(crate) struct OwnedEvent(pub(crate) CGEventRef);
impl Drop for OwnedEvent {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { CFRelease(self.0 as CFTypeRef) };
        }
    }
}

pub(crate) struct OwnedSource(pub(crate) CGEventSourceRef);
impl Drop for OwnedSource {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { CFRelease(self.0 as CFTypeRef) };
        }
    }
}

/// Generic owner for any retained CFTypeRef (AXUIElementRef, CFStringRef,
/// CFArrayRef, ...). Calls `CFRelease` on drop.
pub(crate) struct OwnedCf(pub(crate) *const c_void);
impl Drop for OwnedCf {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { CFRelease(self.0 as CFTypeRef) };
        }
    }
}

pub(crate) fn cfstring(s: &str) -> Option<OwnedCf> {
    let c = match std::ffi::CString::new(s) {
        Ok(v) => v,
        Err(_) => return None,
    };
    let p = unsafe {
        CFStringCreateWithCString(std::ptr::null(), c.as_ptr(), KCF_STRING_ENCODING_UTF8)
    };
    if p.is_null() {
        None
    } else {
        Some(OwnedCf(p as *const c_void))
    }
}

pub(crate) fn new_source() -> Result<OwnedSource> {
    let s = unsafe { CGEventSourceCreate(KCG_EVENT_SOURCE_STATE_HID_SYSTEM_STATE) };
    if s.is_null() {
        Err(Error::new(
            Status::GenericFailure,
            "CGEventSourceCreate returned null",
        ))
    } else {
        Ok(OwnedSource(s))
    }
}
