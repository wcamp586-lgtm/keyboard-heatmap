//! Fixed physical-key identities for the ANSI 104-key desktop layout.
//!
//! These identifiers deliberately describe fixed hardware positions. They are
//! not characters, text, or a reconstruction of what a person typed.

// Windows scan-code mapping is unused on macOS, which maps Apple virtual key
// codes instead.
#![cfg_attr(target_os = "macos", allow(dead_code))]

use std::fmt;

/// A fixed key position on a 104-key ANSI keyboard.
///
/// The discriminant is the only key identity persisted by SQLite.  It is
/// intentionally an integer so the database never stores a typed character.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum KeyId {
    Escape = 0,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    PrintScreen,
    ScrollLock,
    Pause,
    Grave,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    Digit0,
    Minus,
    Equal,
    Backspace,
    Insert,
    Home,
    PageUp,
    NumLock,
    NumpadDivide,
    NumpadMultiply,
    NumpadSubtract,
    Tab,
    KeyQ,
    KeyW,
    KeyE,
    KeyR,
    KeyT,
    KeyY,
    KeyU,
    KeyI,
    KeyO,
    KeyP,
    BracketLeft,
    BracketRight,
    Backslash,
    Delete,
    End,
    PageDown,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    CapsLock,
    KeyA,
    KeyS,
    KeyD,
    KeyF,
    KeyG,
    KeyH,
    KeyJ,
    KeyK,
    KeyL,
    Semicolon,
    Quote,
    Enter,
    Numpad4,
    Numpad5,
    Numpad6,
    ShiftLeft,
    KeyZ,
    KeyX,
    KeyC,
    KeyV,
    KeyB,
    KeyN,
    KeyM,
    Comma,
    Period,
    Slash,
    ShiftRight,
    ArrowUp,
    Numpad1,
    Numpad2,
    Numpad3,
    NumpadEnter,
    ControlLeft,
    MetaLeft,
    AltLeft,
    Space,
    AltRight,
    MetaRight,
    ContextMenu,
    ControlRight,
    ArrowLeft,
    ArrowDown,
    ArrowRight,
    Numpad0,
    NumpadDecimal,
}

impl KeyId {
    pub const COUNT: usize = 104;

    pub const ALL: [Self; Self::COUNT] = [
        Self::Escape,
        Self::F1,
        Self::F2,
        Self::F3,
        Self::F4,
        Self::F5,
        Self::F6,
        Self::F7,
        Self::F8,
        Self::F9,
        Self::F10,
        Self::F11,
        Self::F12,
        Self::PrintScreen,
        Self::ScrollLock,
        Self::Pause,
        Self::Grave,
        Self::Digit1,
        Self::Digit2,
        Self::Digit3,
        Self::Digit4,
        Self::Digit5,
        Self::Digit6,
        Self::Digit7,
        Self::Digit8,
        Self::Digit9,
        Self::Digit0,
        Self::Minus,
        Self::Equal,
        Self::Backspace,
        Self::Insert,
        Self::Home,
        Self::PageUp,
        Self::NumLock,
        Self::NumpadDivide,
        Self::NumpadMultiply,
        Self::NumpadSubtract,
        Self::Tab,
        Self::KeyQ,
        Self::KeyW,
        Self::KeyE,
        Self::KeyR,
        Self::KeyT,
        Self::KeyY,
        Self::KeyU,
        Self::KeyI,
        Self::KeyO,
        Self::KeyP,
        Self::BracketLeft,
        Self::BracketRight,
        Self::Backslash,
        Self::Delete,
        Self::End,
        Self::PageDown,
        Self::Numpad7,
        Self::Numpad8,
        Self::Numpad9,
        Self::NumpadAdd,
        Self::CapsLock,
        Self::KeyA,
        Self::KeyS,
        Self::KeyD,
        Self::KeyF,
        Self::KeyG,
        Self::KeyH,
        Self::KeyJ,
        Self::KeyK,
        Self::KeyL,
        Self::Semicolon,
        Self::Quote,
        Self::Enter,
        Self::Numpad4,
        Self::Numpad5,
        Self::Numpad6,
        Self::ShiftLeft,
        Self::KeyZ,
        Self::KeyX,
        Self::KeyC,
        Self::KeyV,
        Self::KeyB,
        Self::KeyN,
        Self::KeyM,
        Self::Comma,
        Self::Period,
        Self::Slash,
        Self::ShiftRight,
        Self::ArrowUp,
        Self::Numpad1,
        Self::Numpad2,
        Self::Numpad3,
        Self::NumpadEnter,
        Self::ControlLeft,
        Self::MetaLeft,
        Self::AltLeft,
        Self::Space,
        Self::AltRight,
        Self::MetaRight,
        Self::ContextMenu,
        Self::ControlRight,
        Self::ArrowLeft,
        Self::ArrowDown,
        Self::ArrowRight,
        Self::Numpad0,
        Self::NumpadDecimal,
    ];

    /// Integer persisted in the local aggregate table.
    pub const fn storage_id(self) -> i64 {
        self as u8 as i64
    }

    pub fn from_storage_id(value: i64) -> Option<Self> {
        let index = usize::try_from(value).ok()?;
        let key = *Self::ALL.get(index)?;
        (key.storage_id() == value).then_some(key)
    }

    /// Stable, frontend-facing label. This is generated after an aggregate is
    /// read; it is never written to the SQLite database.
    pub const fn wire_id(self) -> &'static str {
        match self {
            Self::Escape => "escape",
            Self::F1 => "f1",
            Self::F2 => "f2",
            Self::F3 => "f3",
            Self::F4 => "f4",
            Self::F5 => "f5",
            Self::F6 => "f6",
            Self::F7 => "f7",
            Self::F8 => "f8",
            Self::F9 => "f9",
            Self::F10 => "f10",
            Self::F11 => "f11",
            Self::F12 => "f12",
            Self::PrintScreen => "print_screen",
            Self::ScrollLock => "scroll_lock",
            Self::Pause => "pause",
            Self::Grave => "grave",
            Self::Digit1 => "digit_1",
            Self::Digit2 => "digit_2",
            Self::Digit3 => "digit_3",
            Self::Digit4 => "digit_4",
            Self::Digit5 => "digit_5",
            Self::Digit6 => "digit_6",
            Self::Digit7 => "digit_7",
            Self::Digit8 => "digit_8",
            Self::Digit9 => "digit_9",
            Self::Digit0 => "digit_0",
            Self::Minus => "minus",
            Self::Equal => "equal",
            Self::Backspace => "backspace",
            Self::Insert => "insert",
            Self::Home => "home",
            Self::PageUp => "page_up",
            Self::NumLock => "num_lock",
            Self::NumpadDivide => "numpad_divide",
            Self::NumpadMultiply => "numpad_multiply",
            Self::NumpadSubtract => "numpad_subtract",
            Self::Tab => "tab",
            Self::KeyQ => "key_q",
            Self::KeyW => "key_w",
            Self::KeyE => "key_e",
            Self::KeyR => "key_r",
            Self::KeyT => "key_t",
            Self::KeyY => "key_y",
            Self::KeyU => "key_u",
            Self::KeyI => "key_i",
            Self::KeyO => "key_o",
            Self::KeyP => "key_p",
            Self::BracketLeft => "bracket_left",
            Self::BracketRight => "bracket_right",
            Self::Backslash => "backslash",
            Self::Delete => "delete",
            Self::End => "end",
            Self::PageDown => "page_down",
            Self::Numpad7 => "numpad_7",
            Self::Numpad8 => "numpad_8",
            Self::Numpad9 => "numpad_9",
            Self::NumpadAdd => "numpad_add",
            Self::CapsLock => "caps_lock",
            Self::KeyA => "key_a",
            Self::KeyS => "key_s",
            Self::KeyD => "key_d",
            Self::KeyF => "key_f",
            Self::KeyG => "key_g",
            Self::KeyH => "key_h",
            Self::KeyJ => "key_j",
            Self::KeyK => "key_k",
            Self::KeyL => "key_l",
            Self::Semicolon => "semicolon",
            Self::Quote => "quote",
            Self::Enter => "enter",
            Self::Numpad4 => "numpad_4",
            Self::Numpad5 => "numpad_5",
            Self::Numpad6 => "numpad_6",
            Self::ShiftLeft => "shift_left",
            Self::KeyZ => "key_z",
            Self::KeyX => "key_x",
            Self::KeyC => "key_c",
            Self::KeyV => "key_v",
            Self::KeyB => "key_b",
            Self::KeyN => "key_n",
            Self::KeyM => "key_m",
            Self::Comma => "comma",
            Self::Period => "period",
            Self::Slash => "slash",
            Self::ShiftRight => "shift_right",
            Self::ArrowUp => "arrow_up",
            Self::Numpad1 => "numpad_1",
            Self::Numpad2 => "numpad_2",
            Self::Numpad3 => "numpad_3",
            Self::NumpadEnter => "numpad_enter",
            Self::ControlLeft => "control_left",
            Self::MetaLeft => "meta_left",
            Self::AltLeft => "alt_left",
            Self::Space => "space",
            Self::AltRight => "alt_right",
            Self::MetaRight => "meta_right",
            Self::ContextMenu => "context_menu",
            Self::ControlRight => "control_right",
            Self::ArrowLeft => "arrow_left",
            Self::ArrowDown => "arrow_down",
            Self::ArrowRight => "arrow_right",
            Self::Numpad0 => "numpad_0",
            Self::NumpadDecimal => "numpad_decimal",
        }
    }

    /// Whether a RAWKEYBOARD flags value marks a key release.
    pub const fn is_break(flags: u16) -> bool {
        flags & 0x0001 != 0
    }

    /// Maps a Windows RAWKEYBOARD scan position to the fixed ANSI layout.
    ///
    /// This can map both a make and a break event. The Raw Input receiver uses
    /// it for break events solely to clear a volatile in-memory held-key entry;
    /// it never persists the flags or scan code.
    pub fn map_raw_position(make_code: u16, virtual_key: u16, flags: u16) -> Option<Self> {
        const RI_KEY_E0: u16 = 0x0002;
        const RI_KEY_E1: u16 = 0x0004;
        const VK_NUMLOCK: u16 = 0x90;
        const VK_PAUSE: u16 = 0x13;

        // KEYBOARD_OVERRUN_MAKE_CODE is not a physical key. Ignore it rather
        // than letting a malformed report affect a nearby key's count.
        if make_code == 0xff {
            return None;
        }

        let extended = flags & RI_KEY_E0 != 0;
        let extended_one = flags & RI_KEY_E1 != 0;

        // Pause emits an E1 Ctrl prefix; PrintScreen can emit an E0 Shift
        // prefix. Neither prefix represents a physical user press.
        if (extended_one && make_code == 0x1d) || (extended && make_code == 0x2a) {
            return None;
        }

        if virtual_key == VK_PAUSE || (!extended && make_code == 0x45 && virtual_key != VK_NUMLOCK)
        {
            return Some(Self::Pause);
        }
        if virtual_key == VK_NUMLOCK {
            return Some(Self::NumLock);
        }

        if extended {
            return match make_code {
                0x1c => Some(Self::NumpadEnter),
                0x1d => Some(Self::ControlRight),
                0x35 => Some(Self::NumpadDivide),
                0x37 => Some(Self::PrintScreen),
                0x38 => Some(Self::AltRight),
                0x47 => Some(Self::Home),
                0x48 => Some(Self::ArrowUp),
                0x49 => Some(Self::PageUp),
                0x4b => Some(Self::ArrowLeft),
                0x4d => Some(Self::ArrowRight),
                0x4f => Some(Self::End),
                0x50 => Some(Self::ArrowDown),
                0x51 => Some(Self::PageDown),
                0x52 => Some(Self::Insert),
                0x53 => Some(Self::Delete),
                0x5b => Some(Self::MetaLeft),
                0x5c => Some(Self::MetaRight),
                0x5d => Some(Self::ContextMenu),
                _ => None,
            };
        }

        match make_code {
            0x01 => Some(Self::Escape),
            0x02 => Some(Self::Digit1),
            0x03 => Some(Self::Digit2),
            0x04 => Some(Self::Digit3),
            0x05 => Some(Self::Digit4),
            0x06 => Some(Self::Digit5),
            0x07 => Some(Self::Digit6),
            0x08 => Some(Self::Digit7),
            0x09 => Some(Self::Digit8),
            0x0a => Some(Self::Digit9),
            0x0b => Some(Self::Digit0),
            0x0c => Some(Self::Minus),
            0x0d => Some(Self::Equal),
            0x0e => Some(Self::Backspace),
            0x0f => Some(Self::Tab),
            0x10 => Some(Self::KeyQ),
            0x11 => Some(Self::KeyW),
            0x12 => Some(Self::KeyE),
            0x13 => Some(Self::KeyR),
            0x14 => Some(Self::KeyT),
            0x15 => Some(Self::KeyY),
            0x16 => Some(Self::KeyU),
            0x17 => Some(Self::KeyI),
            0x18 => Some(Self::KeyO),
            0x19 => Some(Self::KeyP),
            0x1a => Some(Self::BracketLeft),
            0x1b => Some(Self::BracketRight),
            0x1c => Some(Self::Enter),
            0x1d => Some(Self::ControlLeft),
            0x1e => Some(Self::KeyA),
            0x1f => Some(Self::KeyS),
            0x20 => Some(Self::KeyD),
            0x21 => Some(Self::KeyF),
            0x22 => Some(Self::KeyG),
            0x23 => Some(Self::KeyH),
            0x24 => Some(Self::KeyJ),
            0x25 => Some(Self::KeyK),
            0x26 => Some(Self::KeyL),
            0x27 => Some(Self::Semicolon),
            0x28 => Some(Self::Quote),
            0x29 => Some(Self::Grave),
            0x2a => Some(Self::ShiftLeft),
            0x2b => Some(Self::Backslash),
            0x2c => Some(Self::KeyZ),
            0x2d => Some(Self::KeyX),
            0x2e => Some(Self::KeyC),
            0x2f => Some(Self::KeyV),
            0x30 => Some(Self::KeyB),
            0x31 => Some(Self::KeyN),
            0x32 => Some(Self::KeyM),
            0x33 => Some(Self::Comma),
            0x34 => Some(Self::Period),
            0x35 => Some(Self::Slash),
            0x36 => Some(Self::ShiftRight),
            0x37 => Some(Self::NumpadMultiply),
            0x38 => Some(Self::AltLeft),
            0x39 => Some(Self::Space),
            0x3a => Some(Self::CapsLock),
            0x3b => Some(Self::F1),
            0x3c => Some(Self::F2),
            0x3d => Some(Self::F3),
            0x3e => Some(Self::F4),
            0x3f => Some(Self::F5),
            0x40 => Some(Self::F6),
            0x41 => Some(Self::F7),
            0x42 => Some(Self::F8),
            0x43 => Some(Self::F9),
            0x44 => Some(Self::F10),
            0x46 => Some(Self::ScrollLock),
            0x47 => Some(Self::Numpad7),
            0x48 => Some(Self::Numpad8),
            0x49 => Some(Self::Numpad9),
            0x4a => Some(Self::NumpadSubtract),
            0x4b => Some(Self::Numpad4),
            0x4c => Some(Self::Numpad5),
            0x4d => Some(Self::Numpad6),
            0x4e => Some(Self::NumpadAdd),
            0x4f => Some(Self::Numpad1),
            0x50 => Some(Self::Numpad2),
            0x51 => Some(Self::Numpad3),
            0x52 => Some(Self::Numpad0),
            0x53 => Some(Self::NumpadDecimal),
            0x57 => Some(Self::F11),
            0x58 => Some(Self::F12),
            _ => None,
        }
    }

    /// Maps a Windows RAWKEYBOARD make event to one of the 104 fixed positions.
    ///
    /// `flags` is RAWKEYBOARD::Flags. Break events are intentionally ignored,
    /// so releasing a key never increments a count.
    pub fn from_raw_input(make_code: u16, virtual_key: u16, flags: u16) -> Option<Self> {
        (!Self::is_break(flags))
            .then(|| Self::map_raw_position(make_code, virtual_key, flags))
            .flatten()
    }

    pub const fn does_not_emit_break(self) -> bool {
        matches!(self, Self::Pause)
    }
}

impl fmt::Display for KeyId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.wire_id())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::KeyId;

    #[test]
    fn fixed_ansi_layout_has_104_unique_keys() {
        let distinct = KeyId::ALL.into_iter().collect::<BTreeSet<_>>();
        assert_eq!(KeyId::ALL.len(), KeyId::COUNT);
        assert_eq!(distinct.len(), 104);
        assert_eq!(KeyId::from_storage_id(103), Some(KeyId::NumpadDecimal));
        assert_eq!(KeyId::from_storage_id(104), None);
    }

    #[test]
    fn raw_input_maps_extended_keys_without_recording_key_releases() {
        assert_eq!(KeyId::from_raw_input(0x1e, 0x41, 0), Some(KeyId::KeyA));
        assert_eq!(
            KeyId::from_raw_input(0x1d, 0x11, 0x0002),
            Some(KeyId::ControlRight)
        );
        assert_eq!(KeyId::from_raw_input(0x1e, 0x41, 0x0001), None);
        assert_eq!(KeyId::from_raw_input(0x2a, 0x10, 0x0002), None);
        assert_eq!(KeyId::from_raw_input(0x56, 0, 0), None);
    }

    #[test]
    fn extended_scan_codes_stay_distinct_from_their_ansi_keypad_peers() {
        assert_eq!(KeyId::from_raw_input(0x1c, 0, 0), Some(KeyId::Enter));
        assert_eq!(
            KeyId::from_raw_input(0x1c, 0, 0x0002),
            Some(KeyId::NumpadEnter)
        );
        assert_eq!(KeyId::from_raw_input(0x35, 0, 0), Some(KeyId::Slash));
        assert_eq!(
            KeyId::from_raw_input(0x35, 0, 0x0002),
            Some(KeyId::NumpadDivide)
        );
        assert_eq!(
            KeyId::from_raw_input(0x37, 0, 0),
            Some(KeyId::NumpadMultiply)
        );
        assert_eq!(
            KeyId::from_raw_input(0x37, 0, 0x0002),
            Some(KeyId::PrintScreen)
        );
        assert_eq!(KeyId::from_raw_input(0x47, 0, 0), Some(KeyId::Numpad7));
        assert_eq!(KeyId::from_raw_input(0x47, 0, 0x0002), Some(KeyId::Home));
    }

    #[test]
    fn every_ansi_104_position_has_one_supported_scan_mapping() {
        macro_rules! normal {
            ($key:expr, $scan:expr) => {
                ($key, $scan, 0_u16, 0_u16)
            };
        }
        macro_rules! extended {
            ($key:expr, $scan:expr) => {
                ($key, $scan, 0_u16, 0x0002_u16)
            };
        }

        // Scan Code Set 1 physical positions only: no text or locale
        // conversion. E0 explicitly distinguishes navigation/right modifiers
        // from their keypad or left-side peers.
        let cases = [
            normal!(KeyId::Escape, 0x01),
            normal!(KeyId::F1, 0x3b),
            normal!(KeyId::F2, 0x3c),
            normal!(KeyId::F3, 0x3d),
            normal!(KeyId::F4, 0x3e),
            normal!(KeyId::F5, 0x3f),
            normal!(KeyId::F6, 0x40),
            normal!(KeyId::F7, 0x41),
            normal!(KeyId::F8, 0x42),
            normal!(KeyId::F9, 0x43),
            normal!(KeyId::F10, 0x44),
            normal!(KeyId::F11, 0x57),
            normal!(KeyId::F12, 0x58),
            extended!(KeyId::PrintScreen, 0x37),
            normal!(KeyId::ScrollLock, 0x46),
            // Pause is the one ANSI position whose physical Raw Input report
            // uses the E1 prefix and must be identified by its virtual key.
            (KeyId::Pause, 0x45, 0x13, 0x0004),
            normal!(KeyId::Grave, 0x29),
            normal!(KeyId::Digit1, 0x02),
            normal!(KeyId::Digit2, 0x03),
            normal!(KeyId::Digit3, 0x04),
            normal!(KeyId::Digit4, 0x05),
            normal!(KeyId::Digit5, 0x06),
            normal!(KeyId::Digit6, 0x07),
            normal!(KeyId::Digit7, 0x08),
            normal!(KeyId::Digit8, 0x09),
            normal!(KeyId::Digit9, 0x0a),
            normal!(KeyId::Digit0, 0x0b),
            normal!(KeyId::Minus, 0x0c),
            normal!(KeyId::Equal, 0x0d),
            normal!(KeyId::Backspace, 0x0e),
            extended!(KeyId::Insert, 0x52),
            extended!(KeyId::Home, 0x47),
            extended!(KeyId::PageUp, 0x49),
            (KeyId::NumLock, 0x45, 0x90, 0),
            extended!(KeyId::NumpadDivide, 0x35),
            normal!(KeyId::NumpadMultiply, 0x37),
            normal!(KeyId::NumpadSubtract, 0x4a),
            normal!(KeyId::Tab, 0x0f),
            normal!(KeyId::KeyQ, 0x10),
            normal!(KeyId::KeyW, 0x11),
            normal!(KeyId::KeyE, 0x12),
            normal!(KeyId::KeyR, 0x13),
            normal!(KeyId::KeyT, 0x14),
            normal!(KeyId::KeyY, 0x15),
            normal!(KeyId::KeyU, 0x16),
            normal!(KeyId::KeyI, 0x17),
            normal!(KeyId::KeyO, 0x18),
            normal!(KeyId::KeyP, 0x19),
            normal!(KeyId::BracketLeft, 0x1a),
            normal!(KeyId::BracketRight, 0x1b),
            normal!(KeyId::Backslash, 0x2b),
            extended!(KeyId::Delete, 0x53),
            extended!(KeyId::End, 0x4f),
            extended!(KeyId::PageDown, 0x51),
            normal!(KeyId::Numpad7, 0x47),
            normal!(KeyId::Numpad8, 0x48),
            normal!(KeyId::Numpad9, 0x49),
            normal!(KeyId::NumpadAdd, 0x4e),
            normal!(KeyId::CapsLock, 0x3a),
            normal!(KeyId::KeyA, 0x1e),
            normal!(KeyId::KeyS, 0x1f),
            normal!(KeyId::KeyD, 0x20),
            normal!(KeyId::KeyF, 0x21),
            normal!(KeyId::KeyG, 0x22),
            normal!(KeyId::KeyH, 0x23),
            normal!(KeyId::KeyJ, 0x24),
            normal!(KeyId::KeyK, 0x25),
            normal!(KeyId::KeyL, 0x26),
            normal!(KeyId::Semicolon, 0x27),
            normal!(KeyId::Quote, 0x28),
            normal!(KeyId::Enter, 0x1c),
            normal!(KeyId::Numpad4, 0x4b),
            normal!(KeyId::Numpad5, 0x4c),
            normal!(KeyId::Numpad6, 0x4d),
            normal!(KeyId::ShiftLeft, 0x2a),
            normal!(KeyId::KeyZ, 0x2c),
            normal!(KeyId::KeyX, 0x2d),
            normal!(KeyId::KeyC, 0x2e),
            normal!(KeyId::KeyV, 0x2f),
            normal!(KeyId::KeyB, 0x30),
            normal!(KeyId::KeyN, 0x31),
            normal!(KeyId::KeyM, 0x32),
            normal!(KeyId::Comma, 0x33),
            normal!(KeyId::Period, 0x34),
            normal!(KeyId::Slash, 0x35),
            normal!(KeyId::ShiftRight, 0x36),
            extended!(KeyId::ArrowUp, 0x48),
            normal!(KeyId::Numpad1, 0x4f),
            normal!(KeyId::Numpad2, 0x50),
            normal!(KeyId::Numpad3, 0x51),
            extended!(KeyId::NumpadEnter, 0x1c),
            normal!(KeyId::ControlLeft, 0x1d),
            extended!(KeyId::MetaLeft, 0x5b),
            normal!(KeyId::AltLeft, 0x38),
            normal!(KeyId::Space, 0x39),
            extended!(KeyId::AltRight, 0x38),
            extended!(KeyId::MetaRight, 0x5c),
            extended!(KeyId::ContextMenu, 0x5d),
            extended!(KeyId::ControlRight, 0x1d),
            extended!(KeyId::ArrowLeft, 0x4b),
            extended!(KeyId::ArrowDown, 0x50),
            extended!(KeyId::ArrowRight, 0x4d),
            normal!(KeyId::Numpad0, 0x52),
            normal!(KeyId::NumpadDecimal, 0x53),
        ];

        assert_eq!(cases.len(), KeyId::COUNT);
        let mut mapped = BTreeSet::new();
        for (expected, make_code, virtual_key, flags) in cases {
            assert_eq!(
                KeyId::from_raw_input(make_code, virtual_key, flags),
                Some(expected),
                "failed mapping for {}",
                expected.wire_id()
            );
            assert!(
                mapped.insert(expected),
                "duplicate test case for {expected}"
            );
        }
        assert_eq!(mapped, KeyId::ALL.into_iter().collect());
    }
}
