use crate::platform::CGKeyCode;

// ---------- Key name → kVK_ map ----------

pub(crate) fn key_to_keycode(name: &str) -> Option<CGKeyCode> {
    // Allow raw numeric keycode passthrough.
    if let Ok(n) = name.parse::<u16>() {
        return Some(n);
    }
    let lower = name.to_ascii_lowercase();
    let code: u16 = match lower.as_str() {
        // Letters (kVK_ANSI_*)
        "a" => 0x00,
        "s" => 0x01,
        "d" => 0x02,
        "f" => 0x03,
        "h" => 0x04,
        "g" => 0x05,
        "z" => 0x06,
        "x" => 0x07,
        "c" => 0x08,
        "v" => 0x09,
        "b" => 0x0B,
        "q" => 0x0C,
        "w" => 0x0D,
        "e" => 0x0E,
        "r" => 0x0F,
        "y" => 0x10,
        "t" => 0x11,
        "1" => 0x12,
        "2" => 0x13,
        "3" => 0x14,
        "4" => 0x15,
        "6" => 0x16,
        "5" => 0x17,
        "=" | "equal" => 0x18,
        "9" => 0x19,
        "7" => 0x1A,
        "-" | "minus" => 0x1B,
        "8" => 0x1C,
        "0" => 0x1D,
        "]" | "rightbracket" => 0x1E,
        "o" => 0x1F,
        "u" => 0x20,
        "[" | "leftbracket" => 0x21,
        "i" => 0x22,
        "p" => 0x23,
        "l" => 0x25,
        "j" => 0x26,
        "'" | "quote" => 0x27,
        "k" => 0x28,
        ";" | "semicolon" => 0x29,
        "\\" | "backslash" => 0x2A,
        "," | "comma" => 0x2B,
        "/" | "slash" => 0x2C,
        "n" => 0x2D,
        "m" => 0x2E,
        "." | "period" => 0x2F,
        "`" | "grave" => 0x32,

        // Whitespace / control
        "return" | "enter" => 0x24,
        "tab" => 0x30,
        "space" => 0x31,
        "delete" | "backspace" => 0x33,
        "escape" | "esc" => 0x35,
        "forwarddelete" | "fwddelete" | "del" => 0x75,

        // Modifiers (rarely posted alone, but supported for completeness)
        "command" | "cmd" => 0x37,
        "shift" => 0x38,
        "capslock" => 0x39,
        "option" | "alt" => 0x3A,
        "control" | "ctrl" => 0x3B,
        "rightcommand" | "rcmd" => 0x36,
        "rightshift" | "rshift" => 0x3C,
        "rightoption" | "ralt" => 0x3D,
        "rightcontrol" | "rctrl" => 0x3E,
        "function" | "fn" => 0x3F,

        // Navigation
        "home" => 0x73,
        "pageup" => 0x74,
        "end" => 0x77,
        "pagedown" => 0x79,
        "left" | "leftarrow" => 0x7B,
        "right" | "rightarrow" => 0x7C,
        "down" | "downarrow" => 0x7D,
        "up" | "uparrow" => 0x7E,
        "help" => 0x72,

        // Function row
        "f1" => 0x7A,
        "f2" => 0x78,
        "f3" => 0x63,
        "f4" => 0x76,
        "f5" => 0x60,
        "f6" => 0x61,
        "f7" => 0x62,
        "f8" => 0x64,
        "f9" => 0x65,
        "f10" => 0x6D,
        "f11" => 0x67,
        "f12" => 0x6F,
        "f13" => 0x69,
        "f14" => 0x6B,
        "f15" => 0x71,
        "f16" => 0x6A,
        "f17" => 0x40,
        "f18" => 0x4F,
        "f19" => 0x50,
        "f20" => 0x5A,

        _ => return None,
    };
    Some(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_map_known_keys() {
        assert_eq!(key_to_keycode("return"), Some(0x24));
        assert_eq!(key_to_keycode("Return"), Some(0x24));
        assert_eq!(key_to_keycode("escape"), Some(0x35));
        assert_eq!(key_to_keycode("ESC"), Some(0x35));
        assert_eq!(key_to_keycode("f5"), Some(0x60));
        assert_eq!(key_to_keycode("a"), Some(0x00));
        assert_eq!(key_to_keycode("0"), Some(0x1D));
        assert_eq!(key_to_keycode("36"), Some(36));
        assert_eq!(key_to_keycode("nope"), None);
    }
}
