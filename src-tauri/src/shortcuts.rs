//! Fixed, privacy-preserving shortcut identities.
//!
//! Recognition has a deliberately finite catalogue. Raw Input contributes
//! only physical heatmap positions; the dedicated Windows shortcut observer
//! matches a current modifier state with one fixed trigger and emits a numeric
//! `ShortcutId` or nothing. It never serializes a key sequence, text, a
//! window, an application, or a device identity.

use crate::keymap::KeyId;

/// Built-in shortcuts. The first 18 values are stable so existing local
/// aggregate rows retain their meaning across catalogue upgrades.
///
/// The discriminant is the only shortcut identity persisted by SQLite.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ShortcutId {
    CtrlA = 0,
    CtrlC,
    CtrlX,
    CtrlV,
    CtrlZ,
    CtrlY,
    CtrlS,
    CtrlF,
    CtrlP,
    CtrlN,
    CtrlO,
    CtrlW,
    CtrlT,
    CtrlL,
    CtrlShiftZ,
    CtrlShiftS,
    CtrlShiftN,
    CtrlShiftT,
    CtrlB,
    CtrlI,
    CtrlU,
    CtrlK,
    CtrlH,
    CtrlG,
    CtrlR,
    CtrlD,
    CtrlJ,
    CtrlTab,
    CtrlShiftTab,
    CtrlShiftC,
    CtrlShiftV,
    CtrlShiftP,
    CtrlShiftW,
    CtrlShiftB,
    CtrlShiftDelete,
    CtrlPlus,
    CtrlMinus,
    Ctrl0,
    Ctrl1,
    Ctrl2,
    Ctrl3,
    Ctrl4,
    Ctrl5,
    Ctrl6,
    Ctrl7,
    Ctrl8,
    Ctrl9,
    AltTab,
    AltShiftTab,
    AltF4,
    AltSpace,
    AltEnter,
    AltLeft,
    AltRight,
    WinD,
    WinE,
    WinL,
    WinR,
    WinI,
    WinS,
    WinTab,
    WinV,
    WinX,
    WinA,
    WinN,
    WinP,
    WinM,
    WinHome,
    WinComma,
    WinPeriod,
    WinSpace,
    WinLeft,
    WinRight,
    WinUp,
    WinDown,
    WinShiftS,
    WinShiftM,
    WinShiftLeft,
    WinShiftRight,
    WinShiftUp,
    WinShiftDown,
    WinCtrlD,
    WinCtrlLeft,
    WinCtrlRight,
    WinCtrlF4,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ModifierSet(u8);

impl ModifierSet {
    const NONE: Self = Self(0);
    const CTRL: Self = Self(1 << 0);
    const SHIFT: Self = Self(1 << 1);
    const ALT: Self = Self(1 << 2);
    const WIN: Self = Self(1 << 3);
    const CTRL_SHIFT: Self = Self(Self::CTRL.0 | Self::SHIFT.0);
    const ALT_SHIFT: Self = Self(Self::ALT.0 | Self::SHIFT.0);
    const WIN_SHIFT: Self = Self(Self::WIN.0 | Self::SHIFT.0);
    const WIN_CTRL: Self = Self(Self::WIN.0 | Self::CTRL.0);

    const fn with(self, modifier: Self) -> Self {
        Self(self.0 | modifier.0)
    }

    fn from_held_keys<I>(held_keys: I) -> Self
    where
        I: IntoIterator<Item = KeyId>,
    {
        held_keys.into_iter().fold(Self::NONE, |modifiers, key_id| {
            let modifier = match key_id {
                KeyId::ControlLeft | KeyId::ControlRight => Self::CTRL,
                KeyId::ShiftLeft | KeyId::ShiftRight => Self::SHIFT,
                KeyId::AltLeft | KeyId::AltRight => Self::ALT,
                KeyId::MetaLeft | KeyId::MetaRight => Self::WIN,
                _ => Self::NONE,
            };
            modifiers.with(modifier)
        })
    }
}

#[derive(Clone, Copy)]
struct ShortcutDefinition {
    id: ShortcutId,
    wire_id: &'static str,
    modifiers: ModifierSet,
    trigger: KeyId,
}

macro_rules! shortcut {
    ($id:ident, $wire_id:literal, $modifiers:ident, $trigger:ident) => {
        ShortcutDefinition {
            id: ShortcutId::$id,
            wire_id: $wire_id,
            modifiers: ModifierSet::$modifiers,
            trigger: KeyId::$trigger,
        }
    };
}

/// The complete finite recognition catalogue. Keep old entries first and do
/// not reorder them: their numeric IDs are already stored in local databases.
const SHORTCUT_CATALOG: [ShortcutDefinition; ShortcutId::COUNT] = [
    // Editing and browser basics (stable V1 IDs 0 through 13).
    shortcut!(CtrlA, "ctrl_a", CTRL, KeyA),
    shortcut!(CtrlC, "ctrl_c", CTRL, KeyC),
    shortcut!(CtrlX, "ctrl_x", CTRL, KeyX),
    shortcut!(CtrlV, "ctrl_v", CTRL, KeyV),
    shortcut!(CtrlZ, "ctrl_z", CTRL, KeyZ),
    shortcut!(CtrlY, "ctrl_y", CTRL, KeyY),
    shortcut!(CtrlS, "ctrl_s", CTRL, KeyS),
    shortcut!(CtrlF, "ctrl_f", CTRL, KeyF),
    shortcut!(CtrlP, "ctrl_p", CTRL, KeyP),
    shortcut!(CtrlN, "ctrl_n", CTRL, KeyN),
    shortcut!(CtrlO, "ctrl_o", CTRL, KeyO),
    shortcut!(CtrlW, "ctrl_w", CTRL, KeyW),
    shortcut!(CtrlT, "ctrl_t", CTRL, KeyT),
    shortcut!(CtrlL, "ctrl_l", CTRL, KeyL),
    // Stable V1 Ctrl+Shift IDs 14 through 17.
    shortcut!(CtrlShiftZ, "ctrl_shift_z", CTRL_SHIFT, KeyZ),
    shortcut!(CtrlShiftS, "ctrl_shift_s", CTRL_SHIFT, KeyS),
    shortcut!(CtrlShiftN, "ctrl_shift_n", CTRL_SHIFT, KeyN),
    shortcut!(CtrlShiftT, "ctrl_shift_t", CTRL_SHIFT, KeyT),
    // More common editing, search, and browser commands.
    shortcut!(CtrlB, "ctrl_b", CTRL, KeyB),
    shortcut!(CtrlI, "ctrl_i", CTRL, KeyI),
    shortcut!(CtrlU, "ctrl_u", CTRL, KeyU),
    shortcut!(CtrlK, "ctrl_k", CTRL, KeyK),
    shortcut!(CtrlH, "ctrl_h", CTRL, KeyH),
    shortcut!(CtrlG, "ctrl_g", CTRL, KeyG),
    shortcut!(CtrlR, "ctrl_r", CTRL, KeyR),
    shortcut!(CtrlD, "ctrl_d", CTRL, KeyD),
    shortcut!(CtrlJ, "ctrl_j", CTRL, KeyJ),
    shortcut!(CtrlTab, "ctrl_tab", CTRL, Tab),
    shortcut!(CtrlShiftTab, "ctrl_shift_tab", CTRL_SHIFT, Tab),
    shortcut!(CtrlShiftC, "ctrl_shift_c", CTRL_SHIFT, KeyC),
    shortcut!(CtrlShiftV, "ctrl_shift_v", CTRL_SHIFT, KeyV),
    shortcut!(CtrlShiftP, "ctrl_shift_p", CTRL_SHIFT, KeyP),
    shortcut!(CtrlShiftW, "ctrl_shift_w", CTRL_SHIFT, KeyW),
    shortcut!(CtrlShiftB, "ctrl_shift_b", CTRL_SHIFT, KeyB),
    shortcut!(CtrlShiftDelete, "ctrl_shift_delete", CTRL_SHIFT, Delete),
    // `+` is the physical Equal key with the required Shift modifier.
    shortcut!(CtrlPlus, "ctrl_plus", CTRL_SHIFT, Equal),
    shortcut!(CtrlMinus, "ctrl_minus", CTRL, Minus),
    shortcut!(Ctrl0, "ctrl_0", CTRL, Digit0),
    shortcut!(Ctrl1, "ctrl_1", CTRL, Digit1),
    shortcut!(Ctrl2, "ctrl_2", CTRL, Digit2),
    shortcut!(Ctrl3, "ctrl_3", CTRL, Digit3),
    shortcut!(Ctrl4, "ctrl_4", CTRL, Digit4),
    shortcut!(Ctrl5, "ctrl_5", CTRL, Digit5),
    shortcut!(Ctrl6, "ctrl_6", CTRL, Digit6),
    shortcut!(Ctrl7, "ctrl_7", CTRL, Digit7),
    shortcut!(Ctrl8, "ctrl_8", CTRL, Digit8),
    shortcut!(Ctrl9, "ctrl_9", CTRL, Digit9),
    // Window switching and navigation.
    shortcut!(AltTab, "alt_tab", ALT, Tab),
    shortcut!(AltShiftTab, "alt_shift_tab", ALT_SHIFT, Tab),
    shortcut!(AltF4, "alt_f4", ALT, F4),
    shortcut!(AltSpace, "alt_space", ALT, Space),
    shortcut!(AltEnter, "alt_enter", ALT, Enter),
    shortcut!(AltLeft, "alt_left", ALT, ArrowLeft),
    shortcut!(AltRight, "alt_right", ALT, ArrowRight),
    // Windows system and window-management commands.
    shortcut!(WinD, "win_d", WIN, KeyD),
    shortcut!(WinE, "win_e", WIN, KeyE),
    shortcut!(WinL, "win_l", WIN, KeyL),
    shortcut!(WinR, "win_r", WIN, KeyR),
    shortcut!(WinI, "win_i", WIN, KeyI),
    shortcut!(WinS, "win_s", WIN, KeyS),
    shortcut!(WinTab, "win_tab", WIN, Tab),
    shortcut!(WinV, "win_v", WIN, KeyV),
    shortcut!(WinX, "win_x", WIN, KeyX),
    shortcut!(WinA, "win_a", WIN, KeyA),
    shortcut!(WinN, "win_n", WIN, KeyN),
    shortcut!(WinP, "win_p", WIN, KeyP),
    shortcut!(WinM, "win_m", WIN, KeyM),
    shortcut!(WinHome, "win_home", WIN, Home),
    shortcut!(WinComma, "win_comma", WIN, Comma),
    shortcut!(WinPeriod, "win_period", WIN, Period),
    shortcut!(WinSpace, "win_space", WIN, Space),
    shortcut!(WinLeft, "win_left", WIN, ArrowLeft),
    shortcut!(WinRight, "win_right", WIN, ArrowRight),
    shortcut!(WinUp, "win_up", WIN, ArrowUp),
    shortcut!(WinDown, "win_down", WIN, ArrowDown),
    shortcut!(WinShiftS, "win_shift_s", WIN_SHIFT, KeyS),
    shortcut!(WinShiftM, "win_shift_m", WIN_SHIFT, KeyM),
    shortcut!(WinShiftLeft, "win_shift_left", WIN_SHIFT, ArrowLeft),
    shortcut!(WinShiftRight, "win_shift_right", WIN_SHIFT, ArrowRight),
    shortcut!(WinShiftUp, "win_shift_up", WIN_SHIFT, ArrowUp),
    shortcut!(WinShiftDown, "win_shift_down", WIN_SHIFT, ArrowDown),
    shortcut!(WinCtrlD, "win_ctrl_d", WIN_CTRL, KeyD),
    shortcut!(WinCtrlLeft, "win_ctrl_left", WIN_CTRL, ArrowLeft),
    shortcut!(WinCtrlRight, "win_ctrl_right", WIN_CTRL, ArrowRight),
    shortcut!(WinCtrlF4, "win_ctrl_f4", WIN_CTRL, F4),
];

impl ShortcutId {
    pub const COUNT: usize = 85;

    pub const ALL: [Self; Self::COUNT] = [
        Self::CtrlA,
        Self::CtrlC,
        Self::CtrlX,
        Self::CtrlV,
        Self::CtrlZ,
        Self::CtrlY,
        Self::CtrlS,
        Self::CtrlF,
        Self::CtrlP,
        Self::CtrlN,
        Self::CtrlO,
        Self::CtrlW,
        Self::CtrlT,
        Self::CtrlL,
        Self::CtrlShiftZ,
        Self::CtrlShiftS,
        Self::CtrlShiftN,
        Self::CtrlShiftT,
        Self::CtrlB,
        Self::CtrlI,
        Self::CtrlU,
        Self::CtrlK,
        Self::CtrlH,
        Self::CtrlG,
        Self::CtrlR,
        Self::CtrlD,
        Self::CtrlJ,
        Self::CtrlTab,
        Self::CtrlShiftTab,
        Self::CtrlShiftC,
        Self::CtrlShiftV,
        Self::CtrlShiftP,
        Self::CtrlShiftW,
        Self::CtrlShiftB,
        Self::CtrlShiftDelete,
        Self::CtrlPlus,
        Self::CtrlMinus,
        Self::Ctrl0,
        Self::Ctrl1,
        Self::Ctrl2,
        Self::Ctrl3,
        Self::Ctrl4,
        Self::Ctrl5,
        Self::Ctrl6,
        Self::Ctrl7,
        Self::Ctrl8,
        Self::Ctrl9,
        Self::AltTab,
        Self::AltShiftTab,
        Self::AltF4,
        Self::AltSpace,
        Self::AltEnter,
        Self::AltLeft,
        Self::AltRight,
        Self::WinD,
        Self::WinE,
        Self::WinL,
        Self::WinR,
        Self::WinI,
        Self::WinS,
        Self::WinTab,
        Self::WinV,
        Self::WinX,
        Self::WinA,
        Self::WinN,
        Self::WinP,
        Self::WinM,
        Self::WinHome,
        Self::WinComma,
        Self::WinPeriod,
        Self::WinSpace,
        Self::WinLeft,
        Self::WinRight,
        Self::WinUp,
        Self::WinDown,
        Self::WinShiftS,
        Self::WinShiftM,
        Self::WinShiftLeft,
        Self::WinShiftRight,
        Self::WinShiftUp,
        Self::WinShiftDown,
        Self::WinCtrlD,
        Self::WinCtrlLeft,
        Self::WinCtrlRight,
        Self::WinCtrlF4,
    ];

    /// Integer written to the local aggregate table.
    pub const fn storage_id(self) -> i64 {
        self as u8 as i64
    }

    pub fn from_storage_id(value: i64) -> Option<Self> {
        let index = usize::try_from(value).ok()?;
        let shortcut = *Self::ALL.get(index)?;
        (shortcut.storage_id() == value).then_some(shortcut)
    }

    /// Stable frontend label generated only after an aggregate is read.
    pub fn wire_id(self) -> &'static str {
        self.definition().wire_id
    }

    /// Returns whether a physical key participates as one of the four
    /// recognized modifier families. The trigger is always a non-modifier.
    pub const fn is_modifier_key(key_id: KeyId) -> bool {
        matches!(
            key_id,
            KeyId::ControlLeft
                | KeyId::ControlRight
                | KeyId::ShiftLeft
                | KeyId::ShiftRight
                | KeyId::AltLeft
                | KeyId::AltRight
                | KeyId::MetaLeft
                | KeyId::MetaRight
        )
    }

    /// Matches the one catalogue entry whose trigger and complete normalized
    /// modifier set match the current device state. Any unlisted combination
    /// (including one with an extra modifier) returns `None`.
    pub fn from_pressed_key_with_held_keys<I>(pressed_key: KeyId, held_keys: I) -> Option<Self>
    where
        I: IntoIterator<Item = KeyId>,
    {
        if Self::is_modifier_key(pressed_key) {
            return None;
        }

        let modifiers = ModifierSet::from_held_keys(held_keys);
        SHORTCUT_CATALOG
            .iter()
            .find(|definition| {
                definition.trigger == pressed_key && definition.modifiers == modifiers
            })
            .map(|definition| definition.id)
    }

    /// Matches the fixed catalogue from the normalized modifier state exposed
    /// by the Windows low-level keyboard hook. The hook supplies only these
    /// four transient booleans and one fixed trigger position; no raw event
    /// fields, key order, or text leave that listener.
    pub(crate) fn from_hook_modifier_state(
        pressed_key: KeyId,
        control_down: bool,
        shift_down: bool,
        alt_down: bool,
        win_down: bool,
    ) -> Option<Self> {
        let mut modifiers = [None; 4];
        let mut count = 0;
        if control_down {
            modifiers[count] = Some(KeyId::ControlLeft);
            count += 1;
        }
        if shift_down {
            modifiers[count] = Some(KeyId::ShiftLeft);
            count += 1;
        }
        if alt_down {
            modifiers[count] = Some(KeyId::AltLeft);
            count += 1;
        }
        if win_down {
            modifiers[count] = Some(KeyId::MetaLeft);
        }

        Self::from_pressed_key_with_held_keys(pressed_key, modifiers.into_iter().flatten())
    }

    /// `true` only for positions that can trigger one of the built-in
    /// shortcut aggregates. The low-level hook uses this to suppress repeat
    /// messages without retaining arbitrary key events.
    pub(crate) fn is_catalog_trigger_key(pressed_key: KeyId) -> bool {
        SHORTCUT_CATALOG
            .iter()
            .any(|definition| definition.trigger == pressed_key)
    }

    fn definition(self) -> &'static ShortcutDefinition {
        SHORTCUT_CATALOG
            .iter()
            .find(|definition| definition.id == self)
            .expect("Every fixed shortcut ID must have a catalogue definition")
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{ShortcutId, SHORTCUT_CATALOG};
    use crate::keymap::KeyId;

    #[test]
    fn catalog_has_unique_stable_ids_and_wire_labels() {
        let ids = ShortcutId::ALL.into_iter().collect::<BTreeSet<_>>();
        let wire_ids = SHORTCUT_CATALOG
            .iter()
            .map(|definition| definition.wire_id)
            .collect::<BTreeSet<_>>();

        assert_eq!(ShortcutId::ALL.len(), ShortcutId::COUNT);
        assert_eq!(SHORTCUT_CATALOG.len(), ShortcutId::COUNT);
        assert_eq!(ids.len(), ShortcutId::COUNT);
        assert_eq!(wire_ids.len(), ShortcutId::COUNT);
        assert_eq!(ShortcutId::CtrlA.storage_id(), 0);
        assert_eq!(ShortcutId::CtrlShiftT.storage_id(), 17);
        assert_eq!(ShortcutId::WinCtrlF4.storage_id(), 84);
    }

    #[test]
    fn recognizes_exact_modifier_sets_from_the_fixed_catalog() {
        assert_eq!(
            ShortcutId::from_pressed_key_with_held_keys(
                KeyId::KeyS,
                [KeyId::ControlLeft, KeyId::ShiftRight],
            ),
            Some(ShortcutId::CtrlShiftS)
        );
        assert_eq!(
            ShortcutId::from_pressed_key_with_held_keys(KeyId::KeyC, [KeyId::ControlRight]),
            Some(ShortcutId::CtrlC)
        );
        assert_eq!(
            ShortcutId::from_pressed_key_with_held_keys(KeyId::Tab, [KeyId::AltLeft]),
            Some(ShortcutId::AltTab)
        );
        assert_eq!(
            ShortcutId::from_pressed_key_with_held_keys(
                KeyId::Tab,
                [KeyId::AltLeft, KeyId::ShiftRight],
            ),
            Some(ShortcutId::AltShiftTab)
        );
        assert_eq!(
            ShortcutId::from_pressed_key_with_held_keys(KeyId::KeyD, [KeyId::MetaRight]),
            Some(ShortcutId::WinD)
        );
        assert_eq!(
            ShortcutId::from_pressed_key_with_held_keys(
                KeyId::KeyD,
                [KeyId::MetaLeft, KeyId::ControlLeft],
            ),
            Some(ShortcutId::WinCtrlD)
        );
    }

    #[test]
    fn rejects_extra_modifiers_and_combinations_outside_the_catalog() {
        assert_eq!(
            ShortcutId::from_pressed_key_with_held_keys(
                KeyId::KeyC,
                [KeyId::ControlLeft, KeyId::AltLeft],
            ),
            None
        );
        assert_eq!(
            ShortcutId::from_pressed_key_with_held_keys(
                KeyId::KeyD,
                [KeyId::MetaLeft, KeyId::ControlLeft, KeyId::ShiftLeft],
            ),
            None
        );
        assert_eq!(
            ShortcutId::from_pressed_key_with_held_keys(KeyId::KeyQ, [KeyId::MetaLeft]),
            None
        );
        assert_eq!(
            ShortcutId::from_pressed_key_with_held_keys(KeyId::AltLeft, [KeyId::ControlLeft]),
            None
        );
    }

    #[test]
    fn fixed_numeric_ids_round_trip_without_persisting_labels() {
        assert_eq!(ShortcutId::from_storage_id(0), Some(ShortcutId::CtrlA));
        assert_eq!(
            ShortcutId::from_storage_id(17),
            Some(ShortcutId::CtrlShiftT)
        );
        assert_eq!(ShortcutId::from_storage_id(84), Some(ShortcutId::WinCtrlF4));
        assert_eq!(ShortcutId::from_storage_id(85), None);
        assert_eq!(ShortcutId::WinD.wire_id(), "win_d");
        assert_eq!(ShortcutId::AltF4.wire_id(), "alt_f4");
    }
}
