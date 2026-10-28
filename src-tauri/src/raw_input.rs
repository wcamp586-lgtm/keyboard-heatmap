//! Windows Raw Input receiver.
//!
//! It registers a message-only window for the keyboard usage page with
//! `RIDEV_INPUTSINK`, reads individual `WM_INPUT` messages, maps them
//! immediately to a fixed `KeyId`, then discards the raw payload. It never
//! calls text/layout APIs, never blocks input, and never alters other apps'
//! keyboard handling.

#[cfg(all(not(windows), not(target_os = "macos")))]
use std::sync::Arc;

#[cfg(all(not(windows), not(target_os = "macos")))]
use crate::collector::KeyCollector;

#[cfg(windows)]
mod windows {
    use std::{
        ffi::c_void,
        mem::{self, size_of},
        ptr,
        sync::{
            atomic::{AtomicBool, AtomicIsize, Ordering},
            mpsc, Arc, OnceLock,
        },
        thread,
        time::Duration,
    };

    use windows_sys::Win32::{
        Devices::HumanInterfaceDevice::{HID_USAGE_GENERIC_KEYBOARD, HID_USAGE_PAGE_GENERIC},
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Input::{
                GetRawInputData, RegisterRawInputDevices, HRAWINPUT, RAWINPUT, RAWINPUTDEVICE,
                RAWINPUTHEADER, RIDEV_DEVNOTIFY, RIDEV_INPUTSINK, RID_INPUT, RIM_TYPEKEYBOARD,
            },
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
                PostMessageW, RegisterClassW, GIDC_REMOVAL, HWND_MESSAGE, MSG, WM_INPUT,
                WM_INPUT_DEVICE_CHANGE, WNDCLASSW,
            },
        },
    };

    use crate::{
        collector::KeyCollector,
        keymap::KeyId,
        lifecycle::{LifecycleEvent, LifecycleLogger},
    };

    static COLLECTOR: OnceLock<Arc<KeyCollector>> = OnceLock::new();
    const CONTROL_MESSAGE: u32 = 0x8000 + 0x3b; // WM_APP + a private offset.
    const RAW_INPUT_WINDOW_CLASS: &[u16] = &[
        b'K' as u16,
        b'e' as u16,
        b'y' as u16,
        b'b' as u16,
        b'o' as u16,
        b'a' as u16,
        b'r' as u16,
        b'd' as u16,
        b'H' as u16,
        b'e' as u16,
        b'a' as u16,
        b't' as u16,
        b'm' as u16,
        b'a' as u16,
        b'p' as u16,
        b'R' as u16,
        b'a' as u16,
        b'w' as u16,
        0,
    ];

    struct ListenerState {
        hwnd: AtomicIsize,
        active: AtomicBool,
    }

    struct ControlRequest {
        enabled: bool,
        response: mpsc::SyncSender<Result<(), String>>,
    }

    /// Handle for changing Raw Input registration on its owning message thread.
    /// It carries no keyboard data and exposes no Win32 device handles.
    pub struct RawInputController {
        controls: mpsc::Sender<ControlRequest>,
        state: Arc<ListenerState>,
        lifecycle: LifecycleLogger,
    }

    impl RawInputController {
        pub fn is_active(&self) -> bool {
            self.state.active.load(Ordering::Acquire)
        }

        pub fn set_listening(&self, enabled: bool) -> Result<(), String> {
            if self.is_active() == enabled {
                return Ok(());
            }

            let (response, receive) = mpsc::sync_channel(1);
            if self
                .controls
                .send(ControlRequest { enabled, response })
                .is_err()
            {
                self.lifecycle.record(LifecycleEvent::RawInputFailed);
                return Err("The Raw Input receiver is unavailable".to_owned());
            }
            let hwnd = self.state.hwnd.load(Ordering::Acquire);
            if hwnd == 0 {
                self.lifecycle.record(LifecycleEvent::RawInputFailed);
                return Err("The Raw Input message window is unavailable".to_owned());
            }
            if unsafe { PostMessageW(hwnd as HWND, CONTROL_MESSAGE, 0, 0) } == 0 {
                self.lifecycle.record(LifecycleEvent::RawInputFailed);
                return Err(last_error("Could not signal the Raw Input receiver"));
            }

            match receive.recv_timeout(Duration::from_secs(2)) {
                Ok(result) => {
                    if result.is_err() {
                        self.lifecycle.record(LifecycleEvent::RawInputFailed);
                    }
                    result
                }
                Err(_) => {
                    self.lifecycle.record(LifecycleEvent::RawInputFailed);
                    Err("Timed out updating Raw Input registration".to_owned())
                }
            }
        }
    }

    pub fn start(
        collector: Arc<KeyCollector>,
        lifecycle: LifecycleLogger,
    ) -> Result<RawInputController, String> {
        COLLECTOR
            .set(collector)
            .map_err(|_| "The Raw Input receiver was initialized more than once".to_owned())?;

        let (ready_sender, ready_receiver) = mpsc::sync_channel(1);
        let (control_sender, control_receiver) = mpsc::channel();
        let state = Arc::new(ListenerState {
            hwnd: AtomicIsize::new(0),
            active: AtomicBool::new(false),
        });
        let worker_state = Arc::clone(&state);
        let worker_lifecycle = lifecycle.clone();
        let thread = thread::Builder::new()
            .name("keyboard-heatmap-raw-input".to_owned())
            .spawn(move || {
                run_message_window(
                    ready_sender,
                    control_receiver,
                    worker_state,
                    worker_lifecycle,
                )
            })
            .map_err(|error| format!("Could not start Raw Input thread: {error}"));
        if let Err(error) = thread {
            lifecycle.record(LifecycleEvent::RawInputFailed);
            return Err(error);
        }

        match ready_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(Ok(())) => {}
            Ok(Err(error)) => return Err(error),
            Err(_) => {
                lifecycle.record(LifecycleEvent::RawInputFailed);
                return Err("Timed out while starting the Raw Input receiver".to_owned());
            }
        }

        Ok(RawInputController {
            controls: control_sender,
            state,
            lifecycle,
        })
    }

    fn run_message_window(
        ready_sender: mpsc::SyncSender<Result<(), String>>,
        controls: mpsc::Receiver<ControlRequest>,
        state: Arc<ListenerState>,
        lifecycle: LifecycleLogger,
    ) {
        let result = unsafe { create_and_register_window() };
        let hwnd = match result {
            Ok(hwnd) => {
                state.hwnd.store(hwnd as isize, Ordering::Release);
                state.active.store(true, Ordering::Release);
                let _ = ready_sender.send(Ok(()));
                hwnd
            }
            Err(error) => {
                lifecycle.record(LifecycleEvent::RawInputFailed);
                let _ = ready_sender.send(Err(error));
                return;
            }
        };

        // Standard, one-message-at-a-time Raw Input processing. We do not use
        // buffered input because this listener also receives device-removal
        // notifications for volatile held-key cleanup.
        unsafe {
            let mut message: MSG = mem::zeroed();
            while GetMessageW(&mut message, ptr::null_mut(), 0, 0) > 0 {
                if message.message == CONTROL_MESSAGE {
                    process_control_requests(hwnd, &controls, &state, &lifecycle);
                } else {
                    // Do not call TranslateMessage: this process must never
                    // produce or inspect WM_CHAR text. Dispatching only gives
                    // WM_INPUT to the private message-only window procedure.
                    DispatchMessageW(&message);
                }
            }
            state.active.store(false, Ordering::Release);
            state.hwnd.store(0, Ordering::Release);
            DestroyWindow(hwnd);
        }
        lifecycle.record(LifecycleEvent::RawInputStopped);
    }

    unsafe fn create_and_register_window() -> Result<HWND, String> {
        let instance = GetModuleHandleW(ptr::null());
        if instance.is_null() {
            return Err(last_error("Could not obtain module handle"));
        }

        let mut window_class: WNDCLASSW = mem::zeroed();
        window_class.lpfnWndProc = Some(raw_input_window_proc);
        window_class.hInstance = instance;
        window_class.lpszClassName = RAW_INPUT_WINDOW_CLASS.as_ptr();
        // A zero return can also mean the class is already registered in this
        // process, which is harmless because the class name and procedure are
        // fixed. CreateWindowExW below is the definitive validation.
        let _ = RegisterClassW(&window_class);

        let hwnd = CreateWindowExW(
            0,
            RAW_INPUT_WINDOW_CLASS.as_ptr(),
            RAW_INPUT_WINDOW_CLASS.as_ptr(),
            0,
            0,
            0,
            0,
            0,
            HWND_MESSAGE,
            ptr::null_mut(),
            instance,
            ptr::null(),
        );
        if hwnd.is_null() {
            return Err(last_error("Could not create Raw Input message-only window"));
        }

        register_keyboard_raw_input(hwnd, true)?;

        Ok(hwnd)
    }

    unsafe fn register_keyboard_raw_input(hwnd: HWND, enabled: bool) -> Result<(), String> {
        let keyboard = RAWINPUTDEVICE {
            usUsagePage: HID_USAGE_PAGE_GENERIC,
            usUsage: HID_USAGE_GENERIC_KEYBOARD,
            // INPUTSINK observes keyboard input while the application is in
            // the background. DEVNOTIFY lets us discard ephemeral held-device
            // state when a keyboard disconnects. We intentionally do NOT set
            // NOLEGACY or NOHOTKEYS, so foreground applications receive input
            // normally and no keystroke is blocked or modified. Pausing uses
            // RIDEV_REMOVE with a null target, meaning Windows stops sending
            // this process keyboard raw-input messages altogether.
            dwFlags: if enabled {
                RIDEV_INPUTSINK | RIDEV_DEVNOTIFY
            } else {
                windows_sys::Win32::UI::Input::RIDEV_REMOVE
            },
            hwndTarget: if enabled { hwnd } else { ptr::null_mut() },
        };
        if RegisterRawInputDevices(&keyboard, 1, size_of::<RAWINPUTDEVICE>() as u32) == 0 {
            return Err(last_error(if enabled {
                "Could not register keyboard Raw Input"
            } else {
                "Could not remove keyboard Raw Input"
            }));
        }
        Ok(())
    }

    fn process_control_requests(
        hwnd: HWND,
        controls: &mpsc::Receiver<ControlRequest>,
        state: &ListenerState,
        lifecycle: &LifecycleLogger,
    ) {
        while let Ok(request) = controls.try_recv() {
            let result = unsafe { register_keyboard_raw_input(hwnd, request.enabled) };
            if result.is_ok() {
                state.active.store(request.enabled, Ordering::Release);
            } else {
                lifecycle.record(LifecycleEvent::RawInputFailed);
            }
            let _ = request.response.send(result);
        }
    }

    unsafe extern "system" fn raw_input_window_proc(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match message {
            WM_INPUT => {
                if let Some(collector) = COLLECTOR.get() {
                    read_raw_keyboard_input(lparam, collector);
                }
            }
            WM_INPUT_DEVICE_CHANGE if wparam == GIDC_REMOVAL as usize => {
                if let Some(collector) = COLLECTOR.get() {
                    // lParam is an opaque raw-device handle. It is immediately
                    // used to clear volatile state and is never persisted.
                    collector.forget_device(lparam as usize);
                }
            }
            _ => {}
        }

        // Raw Input's normal legacy delivery remains intact. DefWindowProcW is
        // also required by Windows for foreground RIM_INPUT cleanup.
        DefWindowProcW(window, message, wparam, lparam)
    }

    unsafe fn read_raw_keyboard_input(lparam: LPARAM, collector: &KeyCollector) {
        let mut input: RAWINPUT = mem::zeroed();
        let mut size = size_of::<RAWINPUT>() as u32;
        let copied = GetRawInputData(
            lparam as HRAWINPUT,
            RID_INPUT,
            &mut input as *mut RAWINPUT as *mut c_void,
            &mut size,
            size_of::<RAWINPUTHEADER>() as u32,
        );
        if copied == u32::MAX || copied == 0 || input.header.dwType != RIM_TYPEKEYBOARD {
            return;
        }

        let keyboard = input.data.keyboard;
        let device_id = input.header.hDevice as usize;

        if KeyId::is_break(keyboard.Flags) {
            // A release is used exclusively to remove the volatile held-key
            // entry, so it may use the broader position mapper.
            let Some(key_id) =
                KeyId::map_raw_position(keyboard.MakeCode, keyboard.VKey, keyboard.Flags)
            else {
                return;
            };
            collector.handle_raw_break(key_id, device_id);
        } else {
            // Make events take the press-only mapping path. It rejects breaks
            // by construction and is the only route that can increment data.
            let Some(key_id) =
                KeyId::from_raw_input(keyboard.MakeCode, keyboard.VKey, keyboard.Flags)
            else {
                return;
            };
            collector.handle_raw_make(key_id, device_id);
        }
    }

    fn last_error(context: &str) -> String {
        format!("{context}: {}", std::io::Error::last_os_error())
    }
}

#[cfg(windows)]
pub use windows::{start, RawInputController};

#[cfg(all(not(windows), not(target_os = "macos")))]
pub struct RawInputController;

#[cfg(all(not(windows), not(target_os = "macos")))]
impl RawInputController {
    pub fn is_active(&self) -> bool {
        false
    }

    pub fn set_listening(&self, _enabled: bool) -> Result<(), String> {
        Err("Keyboard Heatmap V1 supports Windows Raw Input only".to_owned())
    }
}

#[cfg(all(not(windows), not(target_os = "macos")))]
pub fn start(
    _collector: Arc<KeyCollector>,
    _lifecycle: crate::lifecycle::LifecycleLogger,
) -> Result<RawInputController, String> {
    Err("Keyboard Heatmap V1 supports Windows Raw Input only".to_owned())
}

#[cfg(target_os = "macos")]
mod macos {
    use std::{
        ffi::c_void,
        ptr,
        sync::{
            atomic::{AtomicBool, AtomicPtr, Ordering},
            Arc, Mutex,
        },
        thread,
    };

    use core_foundation::base::TCFType;
    use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
    use core_graphics::event::{
        CGEvent, CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions,
        CGEventTapPlacement, CGEventType, EventField,
    };

    use crate::{
        collector::KeyCollector,
        keymap::KeyId,
        lifecycle::{LifecycleEvent, LifecycleLogger},
        shortcuts::ShortcutId,
    };

    /// All physical keyboards share one opaque device slot on macOS; the
    /// collector only uses it to suppress auto-repeat events.
    const DEVICE_ID: usize = 0;

    extern "C" {
        fn CGEventTapEnable(tap: *mut c_void, enable: bool) -> bool;
        fn CGPreflightListenEventAccess() -> bool;
        fn CGRequestListenEventAccess() -> bool;
    }

    static TAP_PORT: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());

    pub struct RawInputController {
        active: Arc<AtomicBool>,
        listening: Arc<AtomicBool>,
    }

    impl RawInputController {
        pub fn is_active(&self) -> bool {
            self.active.load(Ordering::Acquire)
        }

        /// The process-wide event tap stays installed; pausing closes the
        /// in-memory intake gate so observed events are dropped immediately.
        pub fn set_listening(&self, enabled: bool) -> Result<(), String> {
            self.listening.store(enabled, Ordering::Release);
            Ok(())
        }
    }

    pub fn start(
        collector: Arc<KeyCollector>,
        lifecycle: LifecycleLogger,
    ) -> Result<RawInputController, String> {
        let active = Arc::new(AtomicBool::new(false));
        let listening = Arc::new(AtomicBool::new(true));
        let thread_active = Arc::clone(&active);
        let thread_listening = Arc::clone(&listening);
        thread::Builder::new()
            .name("keyboard-heatmap-macos-tap".to_owned())
            .spawn(move || run_tap(collector, lifecycle, thread_active, thread_listening))
            .map_err(|error| format!("Could not start the macOS input thread: {error}"))?;
        Ok(RawInputController { active, listening })
    }

    struct TapState {
        collector: Arc<KeyCollector>,
        listening: Arc<AtomicBool>,
        modifiers: Mutex<Modifiers>,
    }

    #[derive(Clone, Copy, Default)]
    struct Modifiers {
        command: bool,
        control: bool,
        shift: bool,
        option: bool,
    }

    fn run_tap(
        collector: Arc<KeyCollector>,
        lifecycle: LifecycleLogger,
        active: Arc<AtomicBool>,
        listening: Arc<AtomicBool>,
    ) {
        // An ad-hoc signed build loses its Input Monitoring grant whenever the
        // binary is rebuilt; macOS then keeps accepting the tap but delivers
        // no events, so the UI would claim "collecting" while nothing arrives.
        // Ask again on start instead of failing silently.
        if !input_monitoring_granted() {
            lifecycle.record(LifecycleEvent::RawInputFailed);
            open_input_monitoring_settings();
            return;
        }

        let state = Arc::new(TapState {
            collector,
            listening,
            modifiers: Mutex::new(Modifiers::default()),
        });
        let callback_state = Arc::clone(&state);

        let tap = CGEventTap::new(
            CGEventTapLocation::HID,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::ListenOnly,
            vec![
                CGEventType::KeyDown,
                CGEventType::KeyUp,
                CGEventType::FlagsChanged,
            ],
            move |_proxy, event_type, event| {
                if matches!(
                    event_type,
                    CGEventType::TapDisabledByTimeout | CGEventType::TapDisabledByUserInput
                ) {
                    // shortcut: a tap disabled repeatedly is not rebuilt, only
                    // re-enabled; a persistent failure needs an app restart.
                    let port = TAP_PORT.load(Ordering::Acquire);
                    if !port.is_null() {
                        unsafe { CGEventTapEnable(port, true) };
                    }
                } else {
                    handle_event(&callback_state, event_type, event);
                }
                None
            },
        );

        let Ok(tap) = tap else {
            lifecycle.record(LifecycleEvent::RawInputFailed);
            open_input_monitoring_settings();
            return;
        };

        let Ok(source) = tap.mach_port.create_runloop_source(0) else {
            lifecycle.record(LifecycleEvent::RawInputFailed);
            return;
        };

        TAP_PORT.store(
            tap.mach_port.as_concrete_TypeRef().cast(),
            Ordering::Release,
        );
        let run_loop = CFRunLoop::get_current();
        unsafe {
            run_loop.add_source(&source, kCFRunLoopCommonModes);
        }
        tap.enable();
        active.store(true, Ordering::Release);
        CFRunLoop::run_current();
    }

    /// Sends the user to the Input Monitoring pane when the tap cannot be
    /// created because the permission has not been granted yet.
    fn open_input_monitoring_settings() {
        let _ = std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent")
            .spawn();
    }

    /// Returns `true` once Input Monitoring is granted. When it has not been
    /// asked for yet, the system prompt is shown and its answer is returned.
    fn input_monitoring_granted() -> bool {
        unsafe { CGPreflightListenEventAccess() || CGRequestListenEventAccess() }
    }

    fn handle_event(state: &TapState, event_type: CGEventType, event: &CGEvent) {
        if !state.listening.load(Ordering::Acquire) {
            return;
        }
        let keycode = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE) as u16;
        match event_type {
            CGEventType::FlagsChanged => handle_flags_changed(state, event, keycode),
            CGEventType::KeyDown => handle_key_down(state, keycode),
            CGEventType::KeyUp => {
                if let Some(key_id) = map_keycode(keycode) {
                    state.collector.handle_raw_break(key_id, DEVICE_ID);
                }
            }
            _ => {}
        }
    }

    fn handle_flags_changed(state: &TapState, event: &CGEvent, keycode: u16) {
        let Some(key_id) = map_modifier_keycode(keycode) else {
            return;
        };
        let flags = event.get_flags();
        let pressed = match key_id {
            KeyId::MetaLeft | KeyId::MetaRight => flags.contains(CGEventFlags::CGEventFlagCommand),
            KeyId::ControlLeft | KeyId::ControlRight => {
                flags.contains(CGEventFlags::CGEventFlagControl)
            }
            KeyId::ShiftLeft | KeyId::ShiftRight => flags.contains(CGEventFlags::CGEventFlagShift),
            KeyId::AltLeft | KeyId::AltRight => flags.contains(CGEventFlags::CGEventFlagAlternate),
            KeyId::CapsLock => flags.contains(CGEventFlags::CGEventFlagAlphaShift),
            _ => return,
        };

        {
            let mut modifiers = state
                .modifiers
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            match key_id {
                KeyId::MetaLeft | KeyId::MetaRight => modifiers.command = pressed,
                KeyId::ControlLeft | KeyId::ControlRight => modifiers.control = pressed,
                KeyId::ShiftLeft | KeyId::ShiftRight => modifiers.shift = pressed,
                KeyId::AltLeft | KeyId::AltRight => modifiers.option = pressed,
                _ => {}
            }
        }

        if pressed {
            state.collector.handle_raw_make(key_id, DEVICE_ID);
        } else {
            state.collector.handle_raw_break(key_id, DEVICE_ID);
        }
    }

    fn handle_key_down(state: &TapState, keycode: u16) {
        let Some(key_id) = map_keycode(keycode) else {
            return;
        };
        if map_modifier_keycode(keycode).is_some() {
            // Modifier transitions are recorded once, by `FlagsChanged`.
            return;
        }
        state.collector.handle_raw_make(key_id, DEVICE_ID);

        if !ShortcutId::is_catalog_trigger_key(key_id) {
            return;
        }
        let modifiers = *state
            .modifiers
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let mut held = Vec::with_capacity(3);
        // The macOS Command key stands in for Windows' Ctrl here, so ⌘C / ⌘V /
        // ⌘S still land in the existing fixed catalogue. Ctrl is folded into
        // the same bit because it is rarely used alone on a Mac.
        if modifiers.command || modifiers.control {
            held.push(KeyId::ControlLeft);
        }
        if modifiers.shift {
            held.push(KeyId::ShiftLeft);
        }
        if modifiers.option {
            held.push(KeyId::AltLeft);
        }
        if let Some(shortcut_id) = ShortcutId::from_pressed_key_with_held_keys(key_id, held) {
            state.collector.handle_shortcut_make(shortcut_id);
        }
    }

    fn map_modifier_keycode(code: u16) -> Option<KeyId> {
        Some(match code {
            0x36 => KeyId::MetaRight,
            0x37 => KeyId::MetaLeft,
            0x38 => KeyId::ShiftLeft,
            0x39 => KeyId::CapsLock,
            0x3A => KeyId::AltLeft,
            0x3B => KeyId::ControlLeft,
            0x3C => KeyId::ShiftRight,
            0x3D => KeyId::AltRight,
            0x3E => KeyId::ControlRight,
            _ => return None,
        })
    }

    /// Maps Apple virtual key codes to the fixed ANSI 104 positions. Positions
    /// that have no key on a Mac keyboard (PrintScreen, ScrollLock, Pause,
    /// Insert, ContextMenu) stay unmapped and therefore read as zero.
    fn map_keycode(code: u16) -> Option<KeyId> {
        Some(match code {
            0x00 => KeyId::KeyA,
            0x01 => KeyId::KeyS,
            0x02 => KeyId::KeyD,
            0x03 => KeyId::KeyF,
            0x04 => KeyId::KeyH,
            0x05 => KeyId::KeyG,
            0x06 => KeyId::KeyZ,
            0x07 => KeyId::KeyX,
            0x08 => KeyId::KeyC,
            0x09 => KeyId::KeyV,
            0x0B => KeyId::KeyB,
            0x0C => KeyId::KeyQ,
            0x0D => KeyId::KeyW,
            0x0E => KeyId::KeyE,
            0x0F => KeyId::KeyR,
            0x10 => KeyId::KeyY,
            0x11 => KeyId::KeyT,
            0x12 => KeyId::Digit1,
            0x13 => KeyId::Digit2,
            0x14 => KeyId::Digit3,
            0x15 => KeyId::Digit4,
            0x16 => KeyId::Digit6,
            0x17 => KeyId::Digit5,
            0x18 => KeyId::Equal,
            0x19 => KeyId::Digit9,
            0x1A => KeyId::Digit7,
            0x1B => KeyId::Minus,
            0x1C => KeyId::Digit8,
            0x1D => KeyId::Digit0,
            0x1E => KeyId::BracketRight,
            0x1F => KeyId::KeyO,
            0x20 => KeyId::KeyU,
            0x21 => KeyId::BracketLeft,
            0x22 => KeyId::KeyI,
            0x23 => KeyId::KeyP,
            0x24 => KeyId::Enter,
            0x25 => KeyId::KeyL,
            0x26 => KeyId::KeyJ,
            0x27 => KeyId::Quote,
            0x28 => KeyId::KeyK,
            0x29 => KeyId::Semicolon,
            0x2A => KeyId::Backslash,
            0x2B => KeyId::Comma,
            0x2C => KeyId::Slash,
            0x2D => KeyId::KeyN,
            0x2E => KeyId::KeyM,
            0x2F => KeyId::Period,
            0x30 => KeyId::Tab,
            0x31 => KeyId::Space,
            0x32 => KeyId::Grave,
            0x33 => KeyId::Backspace,
            0x35 => KeyId::Escape,
            0x36 => KeyId::MetaRight,
            0x37 => KeyId::MetaLeft,
            0x38 => KeyId::ShiftLeft,
            0x39 => KeyId::CapsLock,
            0x3A => KeyId::AltLeft,
            0x3B => KeyId::ControlLeft,
            0x3C => KeyId::ShiftRight,
            0x3D => KeyId::AltRight,
            0x3E => KeyId::ControlRight,
            0x41 => KeyId::NumpadDecimal,
            0x43 => KeyId::NumpadMultiply,
            0x45 => KeyId::NumpadAdd,
            0x47 => KeyId::NumLock,
            0x4B => KeyId::NumpadDivide,
            0x4C => KeyId::NumpadEnter,
            0x4E => KeyId::NumpadSubtract,
            0x52 => KeyId::Numpad0,
            0x53 => KeyId::Numpad1,
            0x54 => KeyId::Numpad2,
            0x55 => KeyId::Numpad3,
            0x56 => KeyId::Numpad4,
            0x57 => KeyId::Numpad5,
            0x58 => KeyId::Numpad6,
            0x59 => KeyId::Numpad7,
            0x5B => KeyId::Numpad8,
            0x5C => KeyId::Numpad9,
            0x60 => KeyId::F5,
            0x61 => KeyId::F6,
            0x62 => KeyId::F7,
            0x63 => KeyId::F3,
            0x64 => KeyId::F8,
            0x65 => KeyId::F9,
            0x67 => KeyId::F11,
            0x6D => KeyId::F10,
            0x6F => KeyId::F12,
            0x73 => KeyId::Home,
            0x74 => KeyId::PageUp,
            0x75 => KeyId::Delete,
            0x76 => KeyId::F4,
            0x77 => KeyId::End,
            0x78 => KeyId::F2,
            0x79 => KeyId::PageDown,
            0x7A => KeyId::F1,
            0x7B => KeyId::ArrowLeft,
            0x7C => KeyId::ArrowRight,
            0x7D => KeyId::ArrowDown,
            0x7E => KeyId::ArrowUp,
            _ => return None,
        })
    }
}

#[cfg(target_os = "macos")]
pub use macos::{start, RawInputController};
