use crate::ffi::{
    CGWindowListCopyWindowInfo, CGWindowListOption, K_CG_NULL_WINDOW_ID,
    K_CG_WINDOW_LIST_EXCLUDE_DESKTOP_ELEMENTS, K_CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY,
};
use crate::types::{ListWindowsOptions, MacWindow, MacWindowBounds};
use core_foundation::base::{CFType, TCFType};
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::CFDictionary;
use core_foundation::number::CFNumber;
use core_foundation::string::CFString;
use core_foundation::ConcreteCFType;
use core_foundation_sys::array::{CFArrayGetCount, CFArrayGetValueAtIndex};
use core_foundation_sys::base::CFTypeRef;
use core_foundation_sys::dictionary::CFDictionaryRef;
use napi::bindgen_prelude::*;
use napi_derive::napi;

type Dict = CFDictionary<CFString, CFType>;

fn dict_get<T: TCFType + ConcreteCFType>(dict: &Dict, key: &str) -> Option<T> {
    let key = CFString::new(key);
    let value = dict.find(&key)?;
    value.downcast::<T>()
}

fn dict_get_string(dict: &Dict, key: &str) -> Option<String> {
    dict_get::<CFString>(dict, key).map(|s| s.to_string())
}

fn dict_get_i32(dict: &Dict, key: &str) -> Option<i32> {
    dict_get::<CFNumber>(dict, key).and_then(|n| n.to_i32())
}

fn dict_get_u32(dict: &Dict, key: &str) -> Option<u32> {
    dict_get_i32(dict, key).map(|v| v as u32)
}

fn dict_get_bool(dict: &Dict, key: &str) -> Option<bool> {
    dict_get::<CFBoolean>(dict, key).map(|b| b == CFBoolean::true_value())
}

fn dict_get_f64(dict: &Dict, key: &str) -> Option<f64> {
    dict_get::<CFNumber>(dict, key).and_then(|n| n.to_f64())
}

fn read_bounds(dict: &Dict) -> MacWindowBounds {
    // CGRect is serialised as a nested CFDictionary with X / Y / Width / Height
    // CFNumber keys. The element-typed `CFDictionary<CFString, CFType>` does not
    // implement ConcreteCFType, so we cannot go through `dict_get::<Dict>(...)`.
    // Instead, look up the value as the generic CFType and re-wrap the
    // underlying pointer as a typed dictionary.
    let key = CFString::new("kCGWindowBounds");
    let zero = MacWindowBounds {
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
    };
    let value = match dict.find(&key) {
        Some(v) => v,
        None => return zero,
    };
    let bounds_dict: Dict =
        unsafe { CFDictionary::wrap_under_get_rule(value.as_CFTypeRef() as CFDictionaryRef) };
    MacWindowBounds {
        x: dict_get_f64(&bounds_dict, "X").unwrap_or(0.0),
        y: dict_get_f64(&bounds_dict, "Y").unwrap_or(0.0),
        width: dict_get_f64(&bounds_dict, "Width").unwrap_or(0.0),
        height: dict_get_f64(&bounds_dict, "Height").unwrap_or(0.0),
    }
}

/// List on-screen windows with their owning application name and bounds.
#[napi]
pub fn list_windows(options: Option<ListWindowsOptions>) -> Result<Vec<MacWindow>> {
    let opts = options.unwrap_or(ListWindowsOptions {
        on_screen_only: None,
        exclude_desktop_elements: None,
        include_system_chrome: None,
    });

    let on_screen_only = opts.on_screen_only.unwrap_or(true);
    let exclude_desktop = opts.exclude_desktop_elements.unwrap_or(true);
    let include_chrome = opts.include_system_chrome.unwrap_or(false);

    let mut option_bits: CGWindowListOption = 0;
    if on_screen_only {
        option_bits |= K_CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY;
    }
    if exclude_desktop {
        option_bits |= K_CG_WINDOW_LIST_EXCLUDE_DESKTOP_ELEMENTS;
    }

    let array_ref = unsafe { CGWindowListCopyWindowInfo(option_bits, K_CG_NULL_WINDOW_ID) };
    if array_ref.is_null() {
        return Err(Error::new(
            Status::GenericFailure,
            "CGWindowListCopyWindowInfo returned null. Verify Screen Recording permission.",
        ));
    }

    // SAFETY: CGWindowListCopyWindowInfo returns a +1 retained CFArray. We must
    // release it exactly once. We do that by wrapping the count + each element
    // manually and calling CFRelease on the array at the end.
    let count = unsafe { CFArrayGetCount(array_ref) };
    let mut out: Vec<MacWindow> = Vec::with_capacity(count as usize);

    for i in 0..count {
        let raw: CFTypeRef = unsafe { CFArrayGetValueAtIndex(array_ref, i) as CFTypeRef };
        if raw.is_null() {
            continue;
        }
        let dict: Dict = unsafe { CFDictionary::wrap_under_get_rule(raw as CFDictionaryRef) };

        let layer = dict_get_i32(&dict, "kCGWindowLayer").unwrap_or(0);
        if !include_chrome && layer != 0 {
            continue;
        }

        let owner_name = dict_get_string(&dict, "kCGWindowOwnerName").unwrap_or_default();
        if owner_name.is_empty() {
            continue;
        }

        let window_id = dict_get_u32(&dict, "kCGWindowNumber").unwrap_or(0);
        let owner_pid = dict_get_i32(&dict, "kCGWindowOwnerPID").unwrap_or(0);
        let title = dict_get_string(&dict, "kCGWindowName").filter(|s| !s.is_empty());
        let bounds = read_bounds(&dict);
        let on_screen = dict_get_bool(&dict, "kCGWindowIsOnscreen").unwrap_or(false);

        out.push(MacWindow {
            window_id,
            owner_name,
            owner_pid,
            title,
            bounds,
            layer,
            on_screen,
        });
    }

    // Release the +1 retain from CGWindowListCopyWindowInfo.
    unsafe {
        core_foundation_sys::base::CFRelease(array_ref as CFTypeRef);
    }

    Ok(out)
}
