use crate::platform::*;
use crate::types::*;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::os::raw::{c_char, c_void};

/// Inside the target app, find the window matching `target_window_id` (a
/// CGWindowID) and make it the key window via the AX `AXRaise` action.
/// Returns true if the window was found and raised, false otherwise.
///
/// `AXRaise` is a window-level action — it brings the window to the front of
/// its app's window stack and gives it key focus, but does NOT activate the
/// app at the system level. Combined with `CGEventPostToPid` for the actual
/// keystrokes, this guarantees keyboard input lands in the focused first
/// responder of the intended window without disturbing the user's foreground
/// app.
///
/// Best-effort: silently returns false on any AX error so callers (typically
/// `background_type` / `background_press_key`) can still proceed — sometimes
/// the keystroke will land correctly anyway because the target window was
/// already key, sometimes it won't.
pub(crate) fn ensure_window_key(pid: i32, target_window_id: u32) -> bool {
    let app = unsafe { AXUIElementCreateApplication(pid) };
    if app.is_null() {
        return false;
    }
    let _app_owner = OwnedCf(app as *const c_void);

    let attr_windows = match cfstring("AXWindows") {
        Some(s) => s,
        None => return false,
    };

    let mut windows_value: CFTypeRef = std::ptr::null();
    let err = unsafe {
        AXUIElementCopyAttributeValue(app, attr_windows.0 as CFStringRef, &mut windows_value)
    };
    if err != KAX_ERROR_SUCCESS || windows_value.is_null() {
        return false;
    }
    let windows_owner = OwnedCf(windows_value);
    let windows_array = windows_owner.0 as CFArrayRef;

    let count = unsafe { CFArrayGetCount(windows_array) };
    let mut found_window: AXUIElementRef = std::ptr::null_mut();
    for i in 0..count {
        let elem = unsafe { CFArrayGetValueAtIndex(windows_array, i) as AXUIElementRef };
        if elem.is_null() {
            continue;
        }
        let mut wid: u32 = 0;
        let err = unsafe { _AXUIElementGetWindow(elem, &mut wid) };
        if err == KAX_ERROR_SUCCESS && wid == target_window_id {
            found_window = elem;
            break;
        }
    }

    if found_window.is_null() {
        return false;
    }

    let raise = match cfstring("AXRaise") {
        Some(s) => s,
        None => return false,
    };
    let err = unsafe { AXUIElementPerformAction(found_window, raise.0 as CFStringRef) };
    err == KAX_ERROR_SUCCESS
}

fn ax_copy_attr(element: AXUIElementRef, attribute: &str) -> Option<OwnedCf> {
    let attr = cfstring(attribute)?;
    let mut value: CFTypeRef = std::ptr::null();
    let err = unsafe { AXUIElementCopyAttributeValue(element, attr.0 as CFStringRef, &mut value) };
    if err != KAX_ERROR_SUCCESS || value.is_null() {
        None
    } else {
        Some(OwnedCf(value as *const c_void))
    }
}

fn ax_enable_chromium_accessibility(app: AXUIElementRef) {
    for attribute in ["AXEnhancedUserInterface", "AXManualAccessibility"] {
        if let Some(attr) = cfstring(attribute) {
            unsafe {
                AXUIElementSetAttributeValue(
                    app,
                    attr.0 as CFStringRef,
                    kCFBooleanTrue as CFTypeRef,
                );
            }
        }
    }
}

fn sanitize_ax_text(raw: &str, max_chars: usize) -> Option<String> {
    let mut out = String::new();
    let mut emitted = 0usize;
    let mut seen_non_ws = false;
    let mut pending_space = false;
    let mut truncated = false;

    for ch in raw.chars() {
        if ch.is_whitespace() {
            if seen_non_ws {
                pending_space = true;
            }
            continue;
        }

        if pending_space && !out.is_empty() {
            if emitted >= max_chars {
                truncated = true;
                break;
            }
            out.push(' ');
            emitted += 1;
            pending_space = false;
        }

        if emitted >= max_chars {
            truncated = true;
            break;
        }
        out.push(ch);
        emitted += 1;
        seen_non_ws = true;
    }

    if out.is_empty() {
        None
    } else if truncated {
        Some(format!("{out}..."))
    } else {
        Some(out)
    }
}

fn cfstring_ref_to_string(value: CFStringRef) -> Option<String> {
    if value.is_null() {
        return None;
    }
    let length = unsafe { CFStringGetLength(value) };
    let buffer_size =
        unsafe { CFStringGetMaximumSizeForEncoding(length, KCF_STRING_ENCODING_UTF8) } + 1;
    if buffer_size <= 0 {
        return None;
    }
    let mut buffer = vec![0u8; buffer_size as usize];
    let ok = unsafe {
        CFStringGetCString(
            value,
            buffer.as_mut_ptr() as *mut c_char,
            buffer_size,
            KCF_STRING_ENCODING_UTF8,
        )
    };
    if ok == 0 {
        return None;
    }
    let nul = buffer
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(buffer.len());
    String::from_utf8(buffer[..nul].to_vec()).ok()
}

fn cf_plain_string(value: CFTypeRef) -> Option<String> {
    if value.is_null() {
        return None;
    }
    let string_type = unsafe { CFStringGetTypeID() };
    let value_type = unsafe { CFGetTypeID(value) };
    if value_type == string_type {
        cfstring_ref_to_string(value as CFStringRef)
    } else {
        None
    }
}

fn cf_array_to_strings(value: CFArrayRef) -> Vec<String> {
    if value.is_null() {
        return Vec::new();
    }
    let mut result = Vec::new();
    let count = unsafe { CFArrayGetCount(value) };
    for idx in 0..count {
        let item = unsafe { CFArrayGetValueAtIndex(value, idx) as CFTypeRef };
        if let Some(text) = cf_plain_string(item) {
            result.push(text);
        }
    }
    result
}

fn cf_description_string(value: CFTypeRef, max_chars: usize) -> Option<String> {
    if value.is_null() {
        return None;
    }
    let desc = unsafe { CFCopyDescription(value) };
    if desc.is_null() {
        return None;
    }
    let desc_owner = OwnedCf(desc as *const c_void);
    let cf = cfstring_ref_to_string(desc_owner.0 as CFStringRef)?;
    sanitize_ax_text(&cf, max_chars)
}

fn cf_value_to_string(value: CFTypeRef, max_chars: usize) -> Option<String> {
    if value.is_null() {
        return None;
    }
    if let Some(raw) = cf_plain_string(value) {
        return sanitize_ax_text(&raw, max_chars);
    }
    cf_description_string(value, max_chars)
}

fn cf_value_kind(value: CFTypeRef) -> Option<&'static str> {
    if value.is_null() {
        return None;
    }
    let value_type = unsafe { CFGetTypeID(value) };
    if value_type == unsafe { CFStringGetTypeID() } {
        return Some("string");
    }
    if value_type == unsafe { CFNumberGetTypeID() } {
        return Some("float");
    }
    if value_type == unsafe { CFBooleanGetTypeID() } {
        return Some("bool");
    }
    None
}

fn ax_attr_string(element: AXUIElementRef, attribute: &str, max_chars: usize) -> Option<String> {
    let value = ax_copy_attr(element, attribute)?;
    cf_value_to_string(value.0 as CFTypeRef, max_chars)
}

fn ax_attr_bool(element: AXUIElementRef, attribute: &str) -> Option<bool> {
    let text = ax_attr_string(element, attribute, 8)?;
    match text.as_str() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn ax_attr_value_kind(element: AXUIElementRef, attribute: &str) -> Option<String> {
    let value = ax_copy_attr(element, attribute)?;
    cf_value_kind(value.0 as CFTypeRef).map(str::to_string)
}

fn ax_attr_is_settable(element: AXUIElementRef, attribute: &str) -> Option<bool> {
    let attr = cfstring(attribute)?;
    let mut settable = false;
    let err =
        unsafe { AXUIElementIsAttributeSettable(element, attr.0 as CFStringRef, &mut settable) };
    if err == KAX_ERROR_SUCCESS {
        Some(settable)
    } else {
        None
    }
}

fn ax_attribute_names(element: AXUIElementRef) -> Vec<String> {
    let mut names: CFArrayRef = std::ptr::null();
    let err = unsafe { AXUIElementCopyAttributeNames(element, &mut names) };
    if err != KAX_ERROR_SUCCESS || names.is_null() {
        return Vec::new();
    }
    let names_owner = OwnedCf(names as CFTypeRef);
    cf_array_to_strings(names_owner.0 as CFArrayRef)
}

fn ax_action_names(element: AXUIElementRef) -> Vec<String> {
    let mut names: CFArrayRef = std::ptr::null();
    let err = unsafe { AXUIElementCopyActionNames(element, &mut names) };
    if err != KAX_ERROR_SUCCESS || names.is_null() {
        return Vec::new();
    }
    let names_owner = OwnedCf(names as CFTypeRef);
    cf_array_to_strings(names_owner.0 as CFArrayRef)
}

fn ax_attr_point(element: AXUIElementRef, attribute: &str) -> Option<Point> {
    let value = ax_copy_attr(element, attribute)?;
    if unsafe { AXValueGetType(value.0 as AXValueRef) } != KAX_VALUE_CGPOINT_TYPE {
        return None;
    }
    let mut point = CGPoint { x: 0.0, y: 0.0 };
    let ok = unsafe {
        AXValueGetValue(
            value.0 as AXValueRef,
            KAX_VALUE_CGPOINT_TYPE,
            &mut point as *mut CGPoint as *mut c_void,
        )
    };
    if !ok {
        return None;
    }
    Some(Point {
        x: point.x,
        y: point.y,
    })
}

fn ax_attr_size(element: AXUIElementRef, attribute: &str) -> Option<AccessibilitySize> {
    let value = ax_copy_attr(element, attribute)?;
    if unsafe { AXValueGetType(value.0 as AXValueRef) } != KAX_VALUE_CGSIZE_TYPE {
        return None;
    }
    let mut size = CGSize {
        width: 0.0,
        height: 0.0,
    };
    let ok = unsafe {
        AXValueGetValue(
            value.0 as AXValueRef,
            KAX_VALUE_CGSIZE_TYPE,
            &mut size as *mut CGSize as *mut c_void,
        )
    };
    if !ok {
        return None;
    }
    Some(AccessibilitySize {
        width: size.width,
        height: size.height,
    })
}

fn frame_from(
    position: &Option<Point>,
    size: &Option<AccessibilitySize>,
) -> Option<AccessibilityFrame> {
    match (position, size) {
        (Some(position), Some(size)) => Some(AccessibilityFrame {
            x: position.x,
            y: position.y,
            width: size.width,
            height: size.height,
        }),
        _ => None,
    }
}

struct AxTreeDumpState {
    nodes: Vec<AccessibilityTreeNode>,
    lines: Vec<String>,
    focused_element: Option<CFTypeRef>,
    focused_summary: Option<AxElementSummary>,
    mode: AxTreeMode,
    focused_element_id: Option<u32>,
    visited: Vec<OwnedCf>,
    next_id: u32,
    max_depth: u32,
    max_nodes: u32,
    truncated: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AxTreeMode {
    Interactive,
    Raw,
}

#[derive(Clone)]
struct AxElementSummary {
    role: String,
    available_actions: Vec<String>,
    title: Option<String>,
    description: Option<String>,
    identifier: Option<String>,
    value: Option<String>,
    selected: Option<bool>,
    frame: Option<AccessibilityFrame>,
}

struct AxChildCandidate {
    element: OwnedCf,
    summary: AxElementSummary,
    keep_as_label: bool,
}

fn push_ax_string(fields: &mut Vec<String>, name: &str, value: Option<&String>) {
    if let Some(value) = value.filter(|value| !value.trim().is_empty()) {
        fields.push(format!("{name}={value:?}"));
    }
}

fn push_ax_bool(fields: &mut Vec<String>, name: &str, value: Option<bool>) {
    if let Some(value) = value {
        fields.push(format!("{name}={value}"));
    }
}

fn ax_line_for_node(node: &AccessibilityTreeNode) -> String {
    let indent = "\t".repeat(node.depth as usize);
    let mut fields: Vec<String> = vec![format!("AXRole={}", node.role)];
    push_ax_string(
        &mut fields,
        "AXRoleDescription",
        node.role_description.as_ref(),
    );
    push_ax_string(&mut fields, "AXSubrole", node.subrole.as_ref());
    push_ax_string(&mut fields, "AXTitle", node.title.as_ref());
    push_ax_string(&mut fields, "AXDescription", node.description.as_ref());
    push_ax_string(&mut fields, "AXValue", node.value.as_ref());
    push_ax_string(&mut fields, "AXValueType", node.value_type.as_ref());
    push_ax_string(&mut fields, "AXURL", node.url.as_ref());
    push_ax_string(&mut fields, "AXIdentifier", node.identifier.as_ref());
    push_ax_string(&mut fields, "AXHelp", node.help.as_ref());
    push_ax_bool(&mut fields, "AXEnabled", node.enabled);
    push_ax_bool(&mut fields, "AXSelected", node.selected);
    push_ax_bool(&mut fields, "AXExpanded", node.expanded);
    push_ax_bool(&mut fields, "AXValueSettable", node.settable);
    if node.focused {
        fields.push("focused=true".to_string());
    }
    if !node.available_actions.is_empty() {
        fields.push(format!("AXActions=[{}]", node.available_actions.join(", ")));
    }

    format!("{indent}[{}] {}", node.id, fields.join(", "))
}

fn render_ax_lines(nodes: &[AccessibilityTreeNode]) -> Vec<String> {
    nodes.iter().map(ax_line_for_node).collect()
}

fn ax_element_seen(elements: &[OwnedCf], element: AXUIElementRef) -> bool {
    if element.is_null() {
        return false;
    }
    elements.iter().any(|seen| {
        seen.0 as AXUIElementRef == element
            || unsafe { CFEqual(seen.0 as CFTypeRef, element as CFTypeRef) }
    })
}

fn ax_element_summary(element: AXUIElementRef) -> AxElementSummary {
    let position = ax_attr_point(element, "AXPosition");
    let size = ax_attr_size(element, "AXSize");
    let frame = frame_from(&position, &size);
    AxElementSummary {
        role: ax_attr_string(element, "AXRole", 80).unwrap_or_else(|| "AXUnknown".to_string()),
        available_actions: ax_action_names(element),
        title: ax_attr_string(element, "AXTitle", 160),
        description: ax_attr_string(element, "AXDescription", 160),
        identifier: ax_attr_string(element, "AXIdentifier", 160),
        value: ax_attr_string(element, "AXValue", 160),
        selected: ax_attr_bool(element, "AXSelected"),
        frame,
    }
}

fn ax_node_text(summary: &AxElementSummary) -> Option<&String> {
    summary
        .title
        .as_ref()
        .or(summary.description.as_ref())
        .or(summary.value.as_ref())
        .filter(|value| !value.trim().is_empty())
}

fn ax_is_label_text(summary: &AxElementSummary) -> bool {
    summary.role == "AXStaticText" && ax_node_text(summary).is_some()
}

fn ax_is_interactive_node(role: &str, available_actions: &[String]) -> bool {
    match role {
        "AXButton"
        | "AXMenuButton"
        | "AXTextField"
        | "AXTextArea"
        | "AXTextView"
        | "AXSearchField"
        | "AXCheckBox"
        | "AXRadioButton"
        | "AXPopUpButton"
        | "AXComboBox"
        | "AXLink"
        | "AXDisclosureTriangle"
        | "AXScrollArea"
        | "AXScrollBar" => true,
        "AXCell" | "AXRow" => !available_actions.is_empty(),
        _ => false,
    }
}

fn ax_is_primary_container_role(role: &str) -> bool {
    matches!(
        role,
        "AXList" | "AXTable" | "AXOutline" | "AXCollection" | "AXScrollArea"
    )
}

fn ax_is_window_container_role(role: &str) -> bool {
    role == "AXWindow"
}

fn ax_is_context_extra_role(role: &str) -> bool {
    matches!(role, "AXRow" | "AXCell" | "AXGroup" | "AXStaticText")
}

fn ax_looks_like_list_item(summary: &AxElementSummary) -> bool {
    let Some(frame) = summary.frame.as_ref() else {
        return false;
    };
    matches!(summary.role.as_str(), "AXRow" | "AXCell" | "AXGroup")
        && frame.width >= 40.0
        && frame.height >= 10.0
        && frame.height <= 180.0
}

fn ax_frames_are_near(
    label_frame: &Option<AccessibilityFrame>,
    control_frame: &Option<AccessibilityFrame>,
) -> bool {
    let (Some(label), Some(control)) = (label_frame, control_frame) else {
        return false;
    };
    let label_center_x = label.x + label.width / 2.0;
    let label_center_y = label.y + label.height / 2.0;
    let control_center_x = control.x + control.width / 2.0;
    let control_center_y = control.y + control.height / 2.0;
    let vertical_overlap =
        label.y < control.y + control.height && control.y < label.y + label.height;
    let horizontal_overlap =
        label.x < control.x + control.width && control.x < label.x + label.width;
    let horizontal_gap = if label.x + label.width < control.x {
        control.x - (label.x + label.width)
    } else if control.x + control.width < label.x {
        label.x - (control.x + control.width)
    } else {
        0.0
    };
    let vertical_gap = if label.y + label.height < control.y {
        control.y - (label.y + label.height)
    } else if control.y + control.height < label.y {
        label.y - (control.y + control.height)
    } else {
        0.0
    };

    (vertical_overlap && horizontal_gap <= 160.0)
        || (horizontal_overlap && vertical_gap <= 48.0)
        || ((label_center_x - control_center_x).abs() <= 220.0
            && (label_center_y - control_center_y).abs() <= 80.0)
}

fn ax_label_is_near_interactive(
    label_index: usize,
    candidates: &[AxChildCandidate],
    parent_is_interactive: bool,
) -> bool {
    if parent_is_interactive {
        return true;
    }
    let label = &candidates[label_index].summary;
    candidates.iter().enumerate().any(|(idx, candidate)| {
        if idx == label_index {
            return false;
        }
        if !ax_is_interactive_node(
            &candidate.summary.role,
            &candidate.summary.available_actions,
        ) {
            return false;
        }
        let adjacent = label_index.abs_diff(idx) <= 2;
        adjacent || ax_frames_are_near(&label.frame, &candidate.summary.frame)
    })
}

fn ax_child_elements(element: AXUIElementRef) -> Vec<OwnedCf> {
    let mut result = Vec::new();
    for attribute in [
        "AXChildren",
        "AXVisibleChildren",
        "AXRows",
        "AXVisibleRows",
        "AXColumns",
        "AXVisibleColumns",
        "AXContents",
        "AXTabs",
        "AXToolbar",
        "AXMenuBar",
        "AXWindows",
        "AXWindow",
        "AXFocusedWindow",
        "AXFocusedUIElement",
    ] {
        for child in ax_related_elements_for_attribute(element, attribute) {
            let child_ref = child.0 as AXUIElementRef;
            if !ax_element_seen(&result, child_ref) {
                result.push(child);
            }
        }
    }
    result
}

fn ax_child_candidates(
    element: AXUIElementRef,
    parent_is_interactive: bool,
) -> Vec<AxChildCandidate> {
    let mut candidates: Vec<AxChildCandidate> = ax_child_elements(element)
        .into_iter()
        .map(|child| {
            let child_ref = child.0 as AXUIElementRef;
            AxChildCandidate {
                element: child,
                summary: ax_element_summary(child_ref),
                keep_as_label: false,
            }
        })
        .collect();

    let label_indexes: Vec<usize> = candidates
        .iter()
        .enumerate()
        .filter_map(|(idx, candidate)| {
            if ax_is_label_text(&candidate.summary) {
                Some(idx)
            } else {
                None
            }
        })
        .collect();
    for idx in label_indexes {
        candidates[idx].keep_as_label =
            ax_label_is_near_interactive(idx, &candidates, parent_is_interactive);
    }

    candidates
}

fn ax_push_unique(values: &mut Vec<String>, value: Option<&String>, max_values: usize) {
    if values.len() >= max_values {
        return;
    }
    let Some(value) = value
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    else {
        return;
    };
    if !values.iter().any(|existing| existing == value) {
        values.push(value.to_string());
    }
}

fn ax_push_summary_text(values: &mut Vec<String>, summary: &AxElementSummary, max_values: usize) {
    ax_push_unique(values, summary.title.as_ref(), max_values);
    ax_push_unique(values, summary.description.as_ref(), max_values);
    ax_push_unique(values, summary.value.as_ref(), max_values);
}

fn ax_collect_descendant_basic_info(
    element: AXUIElementRef,
    visited: &mut Vec<OwnedCf>,
    texts: &mut Vec<String>,
    images: &mut Vec<String>,
    depth: u32,
) {
    if depth >= 3 || (texts.len() + images.len()) >= 6 {
        return;
    }

    for child in ax_child_elements(element) {
        let child_ref = child.0 as AXUIElementRef;
        if child_ref.is_null() || ax_element_seen(visited, child_ref) {
            continue;
        }
        if let Some(retained) = retain_ax_element(child_ref) {
            visited.push(retained);
        }

        let summary = ax_element_summary(child_ref);
        if ax_is_interactive_node(&summary.role, &summary.available_actions)
            && summary.role != "AXTextField"
        {
            continue;
        }

        match summary.role.as_str() {
            "AXStaticText" | "AXTextField" => ax_push_summary_text(texts, &summary, 6),
            "AXImage" => ax_push_summary_text(images, &summary, 4),
            _ => ax_push_summary_text(texts, &summary, 6),
        }

        ax_collect_descendant_basic_info(child_ref, visited, texts, images, depth + 1);
    }
}

fn ax_descendant_basic_description(element: AXUIElementRef) -> Option<String> {
    let mut visited = Vec::new();
    let mut texts = Vec::new();
    let mut images = Vec::new();
    ax_collect_descendant_basic_info(element, &mut visited, &mut texts, &mut images, 0);

    let mut parts = Vec::new();
    if !texts.is_empty() {
        parts.push(format!("text: {}", texts.join(" | ")));
    }
    if !images.is_empty() {
        parts.push(format!("image: {}", images.join(" | ")));
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("; "))
    }
}

fn ax_is_empty_text(value: &Option<String>) -> bool {
    value.as_ref().is_none_or(|value| value.trim().is_empty())
}

fn ax_should_keep_context_node(
    summary: &AxElementSummary,
    focused: bool,
    in_context_region: bool,
    descendant_description: &Option<String>,
) -> bool {
    if !in_context_region || !ax_is_context_extra_role(&summary.role) {
        return false;
    }

    summary.selected.unwrap_or(false)
        || focused
        || !summary.available_actions.is_empty()
        || ax_node_text(summary).is_some()
        || descendant_description.is_some()
        || ax_looks_like_list_item(summary)
}

fn ax_should_add_descendant_description(
    is_interactive: bool,
    keep_context_node: bool,
    role: &str,
) -> bool {
    is_interactive || (keep_context_node && matches!(role, "AXRow" | "AXCell" | "AXGroup"))
}

fn ax_merge_description(
    description: Option<String>,
    descendant_description: Option<String>,
) -> Option<String> {
    match (description, descendant_description) {
        (Some(description), Some(descendant_description))
            if !description.trim().is_empty() && !descendant_description.trim().is_empty() =>
        {
            Some(format!("{description}; {descendant_description}"))
        }
        (Some(description), _) if !description.trim().is_empty() => Some(description),
        (_, Some(descendant_description)) if !descendant_description.trim().is_empty() => {
            Some(descendant_description)
        }
        _ => None,
    }
}

fn normalized_ax_tree_mode(mode: Option<&String>) -> Result<AxTreeMode> {
    match mode
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        None | Some("interactive") | Some("summary") => Ok(AxTreeMode::Interactive),
        Some("raw") | Some("full") => Ok(AxTreeMode::Raw),
        Some(value) => Err(Error::new(
            Status::InvalidArg,
            format!("mode must be \"interactive\" or \"raw\" (got \"{value}\")."),
        )),
    }
}

fn ax_tree_mode_label(mode: AxTreeMode) -> &'static str {
    match mode {
        AxTreeMode::Interactive => "interactive",
        AxTreeMode::Raw => "raw",
    }
}

fn normalized_ax_tree_scope(scope: Option<&String>) -> Result<&'static str> {
    match scope
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        None | Some("focusedWindow") | Some("focused-window") | Some("window") => {
            Ok("focusedWindow")
        }
        Some("app") | Some("application") => Ok("app"),
        Some(value) => Err(Error::new(
            Status::InvalidArg,
            format!("scope must be \"focusedWindow\" or \"app\" (got \"{value}\")."),
        )),
    }
}

fn ax_element_pid(element: AXUIElementRef) -> Option<i32> {
    if element.is_null() {
        return None;
    }
    let mut pid = 0i32;
    let err = unsafe { AXUIElementGetPid(element, &mut pid) };
    if err == KAX_ERROR_SUCCESS && pid > 0 {
        Some(pid)
    } else {
        None
    }
}

fn ax_frames_match(a: &Option<AccessibilityFrame>, b: &Option<AccessibilityFrame>) -> bool {
    let (Some(a), Some(b)) = (a, b) else {
        return false;
    };
    (a.x - b.x).abs() <= 1.0
        && (a.y - b.y).abs() <= 1.0
        && (a.width - b.width).abs() <= 1.0
        && (a.height - b.height).abs() <= 1.0
}

fn ax_matches_focused_element(
    element: AXUIElementRef,
    focused_element: Option<CFTypeRef>,
    focused_summary: Option<&AxElementSummary>,
    summary: &AxElementSummary,
) -> bool {
    let Some(focused_element) = focused_element else {
        return false;
    };
    if unsafe { CFEqual(focused_element, element as CFTypeRef) } {
        return true;
    }
    let Some(focused_summary) = focused_summary else {
        return false;
    };
    if ax_element_pid(element) != ax_element_pid(focused_element as AXUIElementRef) {
        return false;
    }
    if summary.role != focused_summary.role {
        return false;
    }
    if summary.identifier.is_some()
        && focused_summary.identifier.is_some()
        && summary.identifier == focused_summary.identifier
    {
        return true;
    }
    ax_frames_match(&summary.frame, &focused_summary.frame)
}

fn collect_ax_tree(
    element: AXUIElementRef,
    parent_id: Option<u32>,
    depth: u32,
    traversal_depth: u32,
    state: &mut AxTreeDumpState,
    keep_as_label: bool,
    in_context_region: bool,
) {
    if element.is_null() {
        return;
    }
    if ax_element_seen(&state.visited, element) {
        return;
    }
    if let Some(retained) = retain_ax_element(element) {
        state.visited.push(retained);
    }
    if state.nodes.len() >= state.max_nodes as usize {
        state.truncated = true;
        return;
    }

    let role = ax_attr_string(element, "AXRole", 80).unwrap_or_else(|| "AXUnknown".to_string());
    let role_description = ax_attr_string(element, "AXRoleDescription", 120);
    let available_attributes = ax_attribute_names(element);
    let available_actions = ax_action_names(element);
    let position = ax_attr_point(element, "AXPosition");
    let size = ax_attr_size(element, "AXSize");
    let frame = frame_from(&position, &size);
    let summary = AxElementSummary {
        role: role.clone(),
        available_actions: available_actions.clone(),
        title: ax_attr_string(element, "AXTitle", 160),
        description: ax_attr_string(element, "AXDescription", 160),
        identifier: ax_attr_string(element, "AXIdentifier", 160),
        value: ax_attr_string(element, "AXValue", 160),
        selected: ax_attr_bool(element, "AXSelected"),
        frame: frame.clone(),
    };
    let focused = ax_matches_focused_element(
        element,
        state.focused_element,
        state.focused_summary.as_ref(),
        &summary,
    );

    let is_interactive = ax_is_interactive_node(&role, &available_actions);
    let is_primary_container = ax_is_primary_container_role(&role);
    let is_window_container = ax_is_window_container_role(&role);
    let is_context_region = in_context_region || is_primary_container;
    let needs_descendant_description =
        is_interactive || (is_context_region && ax_is_context_extra_role(&role));
    let descendant_description = if needs_descendant_description {
        ax_descendant_basic_description(element)
    } else {
        None
    };
    let keep_context_node = ax_should_keep_context_node(
        &summary,
        focused,
        is_context_region,
        &descendant_description,
    );
    let should_emit = is_interactive
        || state.mode == AxTreeMode::Raw
        || focused
        || is_window_container
        || is_primary_container
        || keep_as_label
        || keep_context_node;
    let mut child_parent_id = parent_id;
    let mut child_depth = depth;
    if should_emit {
        let node_id = state.next_id;
        state.next_id += 1;
        if focused && state.focused_element_id.is_none() {
            state.focused_element_id = Some(node_id);
        }
        let title = summary.title;
        let value = summary.value;
        let mut description = summary.description;
        if ax_should_add_descendant_description(is_interactive, keep_context_node, &role) {
            if keep_context_node && matches!(role.as_str(), "AXRow" | "AXCell" | "AXGroup") {
                description = ax_merge_description(description, descendant_description);
            } else if ax_is_empty_text(&title)
                && ax_is_empty_text(&description)
                && ax_is_empty_text(&value)
            {
                description = descendant_description;
            }
        }
        let node = AccessibilityTreeNode {
            id: node_id,
            parent_id,
            depth,
            role,
            role_description,
            subrole: ax_attr_string(element, "AXSubrole", 120),
            title,
            description,
            identifier: ax_attr_string(element, "AXIdentifier", 160),
            value,
            value_type: ax_attr_value_kind(element, "AXValue"),
            url: ax_attr_string(element, "AXURL", 200),
            help: ax_attr_string(element, "AXHelp", 200),
            enabled: ax_attr_bool(element, "AXEnabled"),
            selected: summary.selected,
            expanded: ax_attr_bool(element, "AXExpanded"),
            settable: ax_attr_is_settable(element, "AXValue"),
            position,
            size,
            frame,
            available_attributes: available_attributes.clone(),
            available_actions: available_actions.clone(),
            focused,
        };

        state.nodes.push(node);
        child_parent_id = Some(node_id);
        child_depth = depth + 1;
    }

    if traversal_depth >= state.max_depth {
        state.truncated = true;
        return;
    }

    let children = ax_child_candidates(element, is_interactive);
    if children.is_empty() {
        return;
    }

    for child in children {
        if state.nodes.len() >= state.max_nodes as usize {
            state.truncated = true;
            break;
        }
        let child_ref = child.element.0 as AXUIElementRef;
        if child_ref.is_null() {
            continue;
        }
        collect_ax_tree(
            child_ref,
            child_parent_id,
            child_depth,
            traversal_depth + 1,
            state,
            child.keep_as_label,
            is_context_region,
        );
    }
}

fn retain_ax_element(element: AXUIElementRef) -> Option<OwnedCf> {
    if element.is_null() {
        return None;
    }
    let retained = unsafe { CFRetain(element as CFTypeRef) };
    if retained.is_null() {
        None
    } else {
        Some(OwnedCf(retained))
    }
}

fn ax_related_elements_for_attribute(element: AXUIElementRef, attribute: &str) -> Vec<OwnedCf> {
    let value = match ax_copy_attr(element, attribute) {
        Some(value) => value,
        None => return Vec::new(),
    };
    let value_type = unsafe { CFGetTypeID(value.0 as CFTypeRef) };
    let ax_element_type = unsafe { AXUIElementGetTypeID() };
    if value_type == ax_element_type {
        return vec![value];
    }
    if value_type != unsafe { CFArrayGetTypeID() } {
        return Vec::new();
    }

    let mut result = Vec::new();
    let count = unsafe { CFArrayGetCount(value.0 as CFArrayRef) };
    for idx in 0..count {
        let item = unsafe { CFArrayGetValueAtIndex(value.0 as CFArrayRef, idx) as CFTypeRef };
        if item.is_null() || unsafe { CFGetTypeID(item) } != ax_element_type {
            continue;
        }
        let retained = unsafe { CFRetain(item) };
        if !retained.is_null() {
            result.push(OwnedCf(retained));
        }
    }
    result
}

fn find_ax_node_by_id(
    element: AXUIElementRef,
    visited: &mut Vec<OwnedCf>,
    next_id: &mut u32,
    target_id: u32,
    traversal_depth: u32,
    max_depth: u32,
    max_nodes: u32,
    keep_as_label: bool,
    focused_element: Option<CFTypeRef>,
    focused_summary: Option<&AxElementSummary>,
    mode: AxTreeMode,
    in_context_region: bool,
) -> Option<OwnedCf> {
    if element.is_null() {
        return None;
    }
    if ax_element_seen(visited, element) {
        return None;
    }
    if let Some(retained) = retain_ax_element(element) {
        visited.push(retained);
    }
    if *next_id >= max_nodes {
        return None;
    }

    let summary = ax_element_summary(element);
    let focused = ax_matches_focused_element(element, focused_element, focused_summary, &summary);
    let is_interactive = ax_is_interactive_node(&summary.role, &summary.available_actions);
    let is_primary_container = ax_is_primary_container_role(&summary.role);
    let is_window_container = ax_is_window_container_role(&summary.role);
    let is_context_region = in_context_region || is_primary_container;
    let descendant_description =
        if is_interactive || (is_context_region && ax_is_context_extra_role(&summary.role)) {
            ax_descendant_basic_description(element)
        } else {
            None
        };
    let keep_context_node = ax_should_keep_context_node(
        &summary,
        focused,
        is_context_region,
        &descendant_description,
    );
    if is_interactive
        || mode == AxTreeMode::Raw
        || focused
        || is_window_container
        || is_primary_container
        || keep_as_label
        || keep_context_node
    {
        let node_id = *next_id;
        *next_id = next_id.saturating_add(1);
        if node_id == target_id {
            return retain_ax_element(element);
        }
    }

    if traversal_depth >= max_depth {
        return None;
    }

    for child in ax_child_candidates(element, is_interactive) {
        let child_ref = child.element.0 as AXUIElementRef;
        if child_ref.is_null() {
            continue;
        }
        if let Some(found) = find_ax_node_by_id(
            child_ref,
            visited,
            next_id,
            target_id,
            traversal_depth + 1,
            max_depth,
            max_nodes,
            child.keep_as_label,
            focused_element,
            focused_summary,
            mode,
            is_context_region,
        ) {
            return Some(found);
        }
    }
    None
}

/// Returns whether the current process is trusted for Accessibility. This is
/// required for the WindowServer to deliver synthetic events to other apps.
#[napi]
pub fn check_accessibility() -> AccessibilityStatus {
    AccessibilityStatus {
        trusted: unsafe { AXIsProcessTrusted() },
    }
}

/// Dump the macOS Accessibility tree for the target app's focused window.
///
/// The output is capped by `max_depth` and `max_nodes` to avoid pathological
/// trees from WebViews / complex Electron apps.
#[napi]
pub fn get_focused_window_accessibility_tree(
    options: FocusedWindowAccessibilityTreeOptions,
) -> Result<FocusedWindowAccessibilityTree> {
    if options.pid <= 0 {
        return Err(Error::new(
            Status::InvalidArg,
            format!("pid must be a positive integer (got {}).", options.pid),
        ));
    }

    let app = unsafe { AXUIElementCreateApplication(options.pid) };
    if app.is_null() {
        return Err(Error::new(
            Status::GenericFailure,
            format!(
                "AXUIElementCreateApplication returned null for pid {}.",
                options.pid
            ),
        ));
    }
    let _app_owner = OwnedCf(app as *const c_void);
    ax_enable_chromium_accessibility(app);

    let scope = normalized_ax_tree_scope(options.scope.as_ref())?;
    let mode = normalized_ax_tree_mode(options.mode.as_ref())?;
    let focused_window = ax_copy_attr(app, "AXFocusedWindow");
    let focused_window_ref = focused_window
        .as_ref()
        .map(|value| value.0 as AXUIElementRef)
        .filter(|value| !value.is_null());
    if scope == "focusedWindow" && focused_window_ref.is_none() {
        return Err(Error::new(
            Status::GenericFailure,
            format!("No focused AX window was found for pid {}.", options.pid),
        ));
    }

    let focused_element = ax_copy_attr(app, "AXFocusedUIElement");
    let focused_summary = focused_element
        .as_ref()
        .map(|value| ax_element_summary(value.0 as AXUIElementRef));
    let window_title = focused_window_ref.and_then(|value| ax_attr_string(value, "AXTitle", 200));
    let root_ref = if scope == "app" {
        app
    } else {
        focused_window_ref.unwrap_or(app)
    };

    let mut state = AxTreeDumpState {
        nodes: Vec::new(),
        lines: Vec::new(),
        focused_element: focused_element.as_ref().map(|value| value.0 as CFTypeRef),
        focused_summary,
        mode,
        focused_element_id: None,
        visited: Vec::new(),
        next_id: 0,
        max_depth: options.max_depth.unwrap_or(200).clamp(1, 200),
        max_nodes: options.max_nodes.unwrap_or(10_000).clamp(1, 10_000),
        truncated: false,
    };
    collect_ax_tree(root_ref, None, 0, 0, &mut state, false, false);
    state.lines = render_ax_lines(&state.nodes);

    let window_label = window_title
        .clone()
        .unwrap_or_else(|| "(untitled)".to_string());
    let mut text = format!(
    "{} accessibility tree\nApp pid: {}\nScope: {}\nFocused window: \"{}\"\nNodes: {}{}\nLimits: maxDepth={} maxNodes={}",
    if mode == AxTreeMode::Raw { "Raw" } else { "Interactive" },
    options.pid,
    scope,
    window_label,
    state.nodes.len(),
    if state.truncated { " (truncated)" } else { "" },
    state.max_depth,
    state.max_nodes
  );
    if !state.lines.is_empty() {
        text.push_str("\n\n");
        text.push_str(&state.lines.join("\n"));
    }
    match state.focused_element_id {
        Some(id) => {
            let focused_label = state
                .nodes
                .iter()
                .find(|node| node.id == id)
                .map(|node| format!("AXRole={}", node.role))
                .unwrap_or_else(|| "element".to_string());
            text.push_str(&format!("\n\nFocused UI element: [{id}] {focused_label}."));
        }
        None => text.push_str("\n\nFocused UI element: not found in the collected tree."),
    }
    if state.truncated {
        text.push_str(&format!(
            "\n\nTree truncated at maxDepth={} / maxNodes={}.",
            state.max_depth, state.max_nodes
        ));
    }

    Ok(FocusedWindowAccessibilityTree {
        pid: options.pid,
        window_title,
        mode: ax_tree_mode_label(mode).to_string(),
        focused_element_id: state.focused_element_id,
        node_count: state.nodes.len() as u32,
        truncated: state.truncated,
        text,
        nodes: state.nodes,
    })
}

/// Perform an accessibility action on a node from the target app's focused-window AX tree.
///
/// Node ids follow the same preorder traversal numbering used by
/// `get_focused_window_accessibility_tree`.
#[napi]
pub fn ax_press_node(options: AxPressNodeOptions) -> Result<AxPressNodeResult> {
    if options.pid <= 0 {
        return Err(Error::new(
            Status::InvalidArg,
            format!("pid must be a positive integer (got {}).", options.pid),
        ));
    }

    let action = options.action.unwrap_or_else(|| "AXPress".to_string());
    let scope = normalized_ax_tree_scope(options.scope.as_ref())?;
    let mode = normalized_ax_tree_mode(options.mode.as_ref())?;
    if action.trim().is_empty() {
        return Err(Error::new(
            Status::InvalidArg,
            "action must be a non-empty accessibility action name.",
        ));
    }

    let app = unsafe { AXUIElementCreateApplication(options.pid) };
    if app.is_null() {
        return Err(Error::new(
            Status::GenericFailure,
            format!(
                "AXUIElementCreateApplication returned null for pid {}.",
                options.pid
            ),
        ));
    }
    let _app_owner = OwnedCf(app as *const c_void);
    ax_enable_chromium_accessibility(app);

    let focused_window = ax_copy_attr(app, "AXFocusedWindow");
    let focused_window_ref = focused_window
        .as_ref()
        .map(|value| value.0 as AXUIElementRef)
        .filter(|value| !value.is_null());
    if scope == "focusedWindow" && focused_window_ref.is_none() {
        return Err(Error::new(
            Status::GenericFailure,
            format!("No focused AX window was found for pid {}.", options.pid),
        ));
    }
    let focused_element = ax_copy_attr(app, "AXFocusedUIElement");
    let focused_summary = focused_element
        .as_ref()
        .map(|value| ax_element_summary(value.0 as AXUIElementRef));
    let root_ref = if scope == "app" {
        app
    } else {
        focused_window_ref.unwrap_or(app)
    };

    let max_depth = options.max_depth.unwrap_or(200).clamp(1, 200);
    let max_nodes = options.max_nodes.unwrap_or(10_000).clamp(1, 10_000);
    let mut next_id = 0u32;
    let mut visited = Vec::new();
    let target = find_ax_node_by_id(
        root_ref,
        &mut visited,
        &mut next_id,
        options.node_id,
        0,
        max_depth,
        max_nodes,
        false,
        focused_element.as_ref().map(|value| value.0 as CFTypeRef),
        focused_summary.as_ref(),
        mode,
        false,
    )
    .ok_or_else(|| {
        Error::new(
            Status::GenericFailure,
            format!(
                "No AX node with id {} was found in the focused window tree for pid {}.",
                options.node_id, options.pid
            ),
        )
    })?;
    let target_ref = target.0 as AXUIElementRef;

    let action_cf = cfstring(&action).ok_or_else(|| {
        Error::new(
            Status::GenericFailure,
            format!("Failed to create CFString for action \"{}\".", action),
        )
    })?;

    let err = unsafe { AXUIElementPerformAction(target_ref, action_cf.0 as CFStringRef) };
    if err != KAX_ERROR_SUCCESS {
        return Err(Error::new(
            Status::GenericFailure,
            format!(
                "AXUIElementPerformAction({}, node {}) failed with AXError {}.",
                action, options.node_id, err
            ),
        ));
    }

    Ok(AxPressNodeResult {
        pid: options.pid,
        node_id: options.node_id,
        action,
        role: ax_attr_string(target_ref, "AXRole", 80).unwrap_or_else(|| "AXUnknown".to_string()),
        title: ax_attr_string(target_ref, "AXTitle", 160),
        identifier: ax_attr_string(target_ref, "AXIdentifier", 160),
        description: ax_attr_string(target_ref, "AXDescription", 160),
    })
}
