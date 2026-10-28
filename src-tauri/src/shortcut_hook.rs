//! Fixed-shortcut listener for Windows-reserved key combinations.
//!
//! Raw Input remains the sole source for physical heatmap positions. Windows
//! can reserve combinations such as Win+D and Alt+Tab before Raw Input reaches
//! an application, so this listener uses `WH_KEYBOARD_LL` as the sole source
//! for *fixed shortcut aggregate IDs*. Its callback only updates atomics and
//! posts one numeric `ShortcutId` to its own message window. It never blocks,
//! writes SQLite, logs, inspects text, or returns a value that could alter the
//! user's keyboard input.

// The Windows hook matcher is untested code on macOS, where the HID tap owns
// shortcut recognition instead.
#![cfg_attr(target_os = "macos", allow(dead_code))]

use crate::{keymap::KeyId, shortcuts::ShortcutId};

const WM_KEYDOWN_MESSAGE: u32 = 0x0100;
const WM_KEYUP_MESSAGE: u32 = 0x0101;
const WM_SYSKEYDOWN_MESSAGE: u32 = 0x0104;
const WM_SYSKEYUP_MESSAGE: u32 = 0x0105;

// `KBDLLHOOKSTRUCT::flags` values. Kept as plain constants so the fixed
// matcher remains unit-testable without retaining a Win32 structure.
const LLKHF_LOWER_IL_INJECTED_FLAG: u32 = 0x0002;
const LLKHF_EXTENDED_FLAG: u32 = 0x0001;
const LLKHF_INJECTED_FLAG: u32 = 0x0010;
const LLKHF_ALTDOWN_FLAG: u32 = 0x0020;

const MOD_LCTRL: u16 = 1 << 0;
const MOD_RCTRL: u16 = 1 << 1;
const MOD_LSHIFT: u16 = 1 << 2;
const MOD_RSHIFT: u16 = 1 << 3;
const MOD_LALT: u16 = 1 << 4;
const MOD_RALT: u16 = 1 << 5;
const MOD_LWIN: u16 = 1 << 6;
const MOD_RWIN: u16 = 1 << 7;

fn is_key_down(message: u32) -> bool {
    matches!(message, WM_KEYDOWN_MESSAGE | WM_SYSKEYDOWN_MESSAGE)
}

fn is_key_up(message: u32) -> bool {
    matches!(message, WM_KEYUP_MESSAGE | WM_SYSKEYUP_MESSAGE)
}

fn is_injected(flags: u32) -> bool {
    flags & (LLKHF_INJECTED_FLAG | LLKHF_LOWER_IL_INJECTED_FLAG) != 0
}

/// Keeps left/right modifier state separate, so releasing one Alt, Ctrl, or
/// Shift key cannot erase the other side while it is still physically held.
fn modifier_bit(virtual_key: u32, scan_code: u32, flags: u32) -> Option<u16> {
    match virtual_key {
        // Hooks sometimes report the generic virtual modifier. The scan code
        // and extended bit preserve left/right state in that case.
        0x11 => Some(if flags & LLKHF_EXTENDED_FLAG != 0 {
            MOD_RCTRL
        } else {
            MOD_LCTRL
        }), // VK_CONTROL
        0xA2 => Some(MOD_LCTRL), // VK_LCONTROL
        0xA3 => Some(MOD_RCTRL), // VK_RCONTROL
        0x10 => Some(if scan_code == 0x36 {
            MOD_RSHIFT
        } else {
            MOD_LSHIFT
        }), // VK_SHIFT
        0xA0 => Some(MOD_LSHIFT), // VK_LSHIFT
        0xA1 => Some(MOD_RSHIFT), // VK_RSHIFT
        0x12 => Some(if flags & LLKHF_EXTENDED_FLAG != 0 {
            MOD_RALT
        } else {
            MOD_LALT
        }), // VK_MENU
        0xA4 => Some(MOD_LALT),  // VK_LMENU
        0xA5 => Some(MOD_RALT),  // VK_RMENU
        0x5B => Some(MOD_LWIN),  // VK_LWIN
        0x5C => Some(MOD_RWIN),  // VK_RWIN
        _ => None,
    }
}

fn modifier_state(bits: u16, flags: u32) -> (bool, bool, bool, bool) {
    let control = bits & (MOD_LCTRL | MOD_RCTRL) != 0;
    let shift = bits & (MOD_LSHIFT | MOD_RSHIFT) != 0;
    // Windows marks Alt state on system-key events. Include that ephemeral
    // flag so Alt+Tab remains detectable even if the Alt-down notification
    // predates a transient listener state reset.
    let alt = bits & (MOD_LALT | MOD_RALT) != 0 || flags & LLKHF_ALTDOWN_FLAG != 0;
    let win = bits & (MOD_LWIN | MOD_RWIN) != 0;
    (control, shift, alt, win)
}

/// Maps only positions used by the fixed shortcut catalogue. It deliberately
/// has no fallback for arbitrary virtual-key codes.
fn catalog_trigger_from_virtual_key(virtual_key: u32) -> Option<KeyId> {
    let key_id = match virtual_key {
        0x09 => KeyId::Tab,        // VK_TAB
        0x0D => KeyId::Enter,      // VK_RETURN
        0x20 => KeyId::Space,      // VK_SPACE
        0x24 => KeyId::Home,       // VK_HOME
        0x25 => KeyId::ArrowLeft,  // VK_LEFT
        0x26 => KeyId::ArrowUp,    // VK_UP
        0x27 => KeyId::ArrowRight, // VK_RIGHT
        0x28 => KeyId::ArrowDown,  // VK_DOWN
        0x2E => KeyId::Delete,     // VK_DELETE
        0x30 => KeyId::Digit0,
        0x31 => KeyId::Digit1,
        0x32 => KeyId::Digit2,
        0x33 => KeyId::Digit3,
        0x34 => KeyId::Digit4,
        0x35 => KeyId::Digit5,
        0x36 => KeyId::Digit6,
        0x37 => KeyId::Digit7,
        0x38 => KeyId::Digit8,
        0x39 => KeyId::Digit9,
        0x41 => KeyId::KeyA,
        0x42 => KeyId::KeyB,
        0x43 => KeyId::KeyC,
        0x44 => KeyId::KeyD,
        0x45 => KeyId::KeyE,
        0x46 => KeyId::KeyF,
        0x47 => KeyId::KeyG,
        0x48 => KeyId::KeyH,
        0x49 => KeyId::KeyI,
        0x4A => KeyId::KeyJ,
        0x4B => KeyId::KeyK,
        0x4C => KeyId::KeyL,
        0x4D => KeyId::KeyM,
        0x4E => KeyId::KeyN,
        0x4F => KeyId::KeyO,
        0x50 => KeyId::KeyP,
        0x51 => KeyId::KeyQ,
        0x52 => KeyId::KeyR,
        0x53 => KeyId::KeyS,
        0x54 => KeyId::KeyT,
        0x55 => KeyId::KeyU,
        0x56 => KeyId::KeyV,
        0x57 => KeyId::KeyW,
        0x58 => KeyId::KeyX,
        0x59 => KeyId::KeyY,
        0x5A => KeyId::KeyZ,
        0x73 => KeyId::F4,     // VK_F4
        0xBB => KeyId::Equal,  // VK_OEM_PLUS
        0xBC => KeyId::Comma,  // VK_OEM_COMMA
        0xBD => KeyId::Minus,  // VK_OEM_MINUS
        0xBE => KeyId::Period, // VK_OEM_PERIOD
        _ => return None,
    };

    ShortcutId::is_catalog_trigger_key(key_id).then_some(key_id)
}

/// Splits the fixed ANSI key number over two atomics. We only call this for a
/// catalogue trigger, so it can never become a retained arbitrary key code.
fn trigger_slot(key_id: KeyId) -> (bool, u64) {
    let index = key_id as u8;
    if index < 64 {
        (false, 1_u64 << index)
    } else {
        (true, 1_u64 << (index - 64))
    }
}

fn match_catalog_shortcut(key_id: KeyId, modifier_bits: u16, flags: u32) -> Option<ShortcutId> {
    let (control, shift, alt, win) = modifier_state(modifier_bits, flags);
    ShortcutId::from_hook_modifier_state(key_id, control, shift, alt, win)
}

#[cfg(test)]
#[derive(Default)]
struct HookMatcher {
    modifier_bits: u16,
    held_trigger_low: u64,
    held_trigger_high: u64,
}

#[cfg(test)]
impl HookMatcher {
    /// A testable equivalent of the callback's fixed matching rules. It only
    /// retains current modifier bits and fixed trigger slots in memory.
    fn observe(&mut self, message: u32, virtual_key: u32, flags: u32) -> Option<ShortcutId> {
        self.observe_with_scan(message, virtual_key, 0, flags)
    }

    fn observe_with_scan(
        &mut self,
        message: u32,
        virtual_key: u32,
        scan_code: u32,
        flags: u32,
    ) -> Option<ShortcutId> {
        if is_injected(flags) {
            return None;
        }

        if is_key_down(message) {
            if let Some(bit) = modifier_bit(virtual_key, scan_code, flags) {
                self.modifier_bits |= bit;
                return None;
            }

            let key_id = catalog_trigger_from_virtual_key(virtual_key)?;
            let (high, bit) = trigger_slot(key_id);
            let held = if high {
                &mut self.held_trigger_high
            } else {
                &mut self.held_trigger_low
            };
            if *held & bit != 0 {
                return None;
            }
            *held |= bit;
            return match_catalog_shortcut(key_id, self.modifier_bits, flags);
        }

        if is_key_up(message) {
            if let Some(bit) = modifier_bit(virtual_key, scan_code, flags) {
                self.modifier_bits &= !bit;
                return None;
            }

            if let Some(key_id) = catalog_trigger_from_virtual_key(virtual_key) {
                let (high, bit) = trigger_slot(key_id);
                let held = if high {
                    &mut self.held_trigger_high
                } else {
                    &mut self.held_trigger_low
                };
                *held &= !bit;
            }
        }

        None
    }
}

#[cfg(windows)]
mod windows {
    use std::{
        mem, ptr,
        sync::{
            atomic::{AtomicBool, AtomicIsize, AtomicU16, AtomicU64, Ordering},
            mpsc, Arc, Mutex, OnceLock,
        },
        thread::{self, JoinHandle},
        time::Duration,
    };

    use windows_sys::Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            CallNextHookEx, CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW,
            GetMessageW, PeekMessageW, PostMessageW, RegisterClassW, SetWindowsHookExW,
            UnhookWindowsHookEx, HHOOK, HWND_MESSAGE, KBDLLHOOKSTRUCT, MSG, PM_REMOVE,
            WH_KEYBOARD_LL, WM_APP, WNDCLASSW,
        },
    };

    use super::{
        catalog_trigger_from_virtual_key, is_injected, is_key_down, is_key_up,
        match_catalog_shortcut, modifier_bit,
    };
    use crate::{
        collector::KeyCollector,
        lifecycle::{LifecycleEvent, LifecycleLogger},
        shortcuts::ShortcutId,
    };

    const CONTROL_MESSAGE: u32 = WM_APP + 0x3C;
    const SHORTCUT_MESSAGE: u32 = WM_APP + 0x3D;
    const HOOK_WINDOW_CLASS: &[u16] = &[
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
        b'S' as u16,
        b'h' as u16,
        b'o' as u16,
        b'r' as u16,
        b't' as u16,
        b'c' as u16,
        b'u' as u16,
        b't' as u16,
        0,
    ];

    struct HookContext {
        collector: Arc<KeyCollector>,
        hwnd: AtomicIsize,
        accepting: AtomicBool,
        cancelled: AtomicBool,
        epoch: AtomicU64,
        modifier_bits: AtomicU16,
        held_trigger_low: AtomicU64,
        held_trigger_high: AtomicU64,
    }

    impl HookContext {
        fn reset_transient_state(&self) {
            self.modifier_bits.store(0, Ordering::Release);
            self.held_trigger_low.store(0, Ordering::Release);
            self.held_trigger_high.store(0, Ordering::Release);
        }

        fn advance_epoch(&self) -> u64 {
            self.epoch.fetch_add(1, Ordering::AcqRel) + 1
        }
    }

    static HOOK_CONTEXT: OnceLock<HookContext> = OnceLock::new();

    struct ListenerState {
        hwnd: AtomicIsize,
        active: AtomicBool,
    }

    enum ControlRequest {
        SetListening {
            enabled: bool,
            response: mpsc::SyncSender<Result<(), String>>,
        },
        Shutdown {
            response: mpsc::SyncSender<()>,
        },
    }

    /// Owns a low-level hook on its dedicated message-pump thread. It exposes
    /// only lifecycle controls, never keyboard events or raw Win32 handles.
    pub struct ShortcutHookController {
        controls: mpsc::Sender<ControlRequest>,
        state: Arc<ListenerState>,
        lifecycle: LifecycleLogger,
        shutdown_requested: AtomicBool,
        worker: Mutex<Option<JoinHandle<()>>>,
    }

    impl ShortcutHookController {
        pub fn is_active(&self) -> bool {
            self.state.active.load(Ordering::Acquire)
        }

        pub fn set_listening(&self, enabled: bool) -> Result<(), String> {
            if self.shutdown_requested.load(Ordering::Acquire) {
                return Err("The fixed-shortcut listener has already stopped".to_owned());
            }
            if HOOK_CONTEXT
                .get()
                .is_some_and(|context| context.cancelled.load(Ordering::Acquire))
            {
                return Err("The fixed-shortcut listener was cancelled".to_owned());
            }
            if self.is_active() == enabled {
                return Ok(());
            }

            let hwnd = self.state.hwnd.load(Ordering::Acquire);
            if hwnd == 0 {
                self.lifecycle.record(LifecycleEvent::ShortcutHookFailed);
                return Err("The fixed-shortcut message window is unavailable".to_owned());
            }

            let (response, receive) = mpsc::sync_channel(1);
            self.controls
                .send(ControlRequest::SetListening { enabled, response })
                .map_err(|_| "The fixed-shortcut listener is unavailable".to_owned())?;
            if unsafe { PostMessageW(hwnd as HWND, CONTROL_MESSAGE, 0, 0) } == 0 {
                self.lifecycle.record(LifecycleEvent::ShortcutHookFailed);
                return Err(last_error("Could not signal the fixed-shortcut listener"));
            }

            match receive.recv_timeout(Duration::from_secs(2)) {
                Ok(result) => result,
                Err(_) => {
                    self.lifecycle.record(LifecycleEvent::ShortcutHookFailed);
                    Err("Timed out updating the fixed-shortcut listener".to_owned())
                }
            }
        }

        pub fn shutdown(&self) {
            if self.shutdown_requested.swap(true, Ordering::AcqRel) {
                return;
            }

            // Make callback and queued-message acceptance impossible before
            // waiting on the owner thread. This avoids a UI-exit hang even if
            // that thread is already unavailable.
            if let Some(context) = HOOK_CONTEXT.get() {
                context.cancelled.store(true, Ordering::Release);
                context.accepting.store(false, Ordering::Release);
                context.advance_epoch();
                context.reset_transient_state();
            }
            self.state.active.store(false, Ordering::Release);

            let hwnd = self.state.hwnd.load(Ordering::Acquire);
            let mut acknowledged_shutdown = false;
            if hwnd != 0 {
                let (response, receive) = mpsc::sync_channel(1);
                if self
                    .controls
                    .send(ControlRequest::Shutdown { response })
                    .is_ok()
                    && unsafe { PostMessageW(hwnd as HWND, CONTROL_MESSAGE, 0, 0) } != 0
                {
                    acknowledged_shutdown = receive.recv_timeout(Duration::from_secs(2)).is_ok();
                }
            }

            if let Some(worker) = self
                .worker
                .lock()
                .expect("Hook worker lock poisoned")
                .take()
            {
                if acknowledged_shutdown {
                    let _ = worker.join();
                } else {
                    // Never make app exit wait indefinitely for a failed
                    // listener thread. Dropping detaches it; the acceptance
                    // gate above already makes its callbacks harmless.
                    self.lifecycle.record(LifecycleEvent::ShortcutHookFailed);
                    drop(worker);
                }
            }
        }
    }

    impl Drop for ShortcutHookController {
        fn drop(&mut self) {
            self.shutdown();
        }
    }

    pub fn start(
        collector: Arc<KeyCollector>,
        lifecycle: LifecycleLogger,
    ) -> Result<ShortcutHookController, String> {
        HOOK_CONTEXT
            .set(HookContext {
                collector,
                hwnd: AtomicIsize::new(0),
                accepting: AtomicBool::new(false),
                cancelled: AtomicBool::new(false),
                epoch: AtomicU64::new(0),
                modifier_bits: AtomicU16::new(0),
                held_trigger_low: AtomicU64::new(0),
                held_trigger_high: AtomicU64::new(0),
            })
            .map_err(|_| "The fixed-shortcut listener was initialized more than once".to_owned())?;

        let (ready_sender, ready_receiver) = mpsc::sync_channel(1);
        let (control_sender, control_receiver) = mpsc::channel();
        let state = Arc::new(ListenerState {
            hwnd: AtomicIsize::new(0),
            active: AtomicBool::new(false),
        });
        let worker_state = Arc::clone(&state);
        let worker_lifecycle = lifecycle.clone();
        let worker = thread::Builder::new()
            .name("keyboard-heatmap-shortcut-hook".to_owned())
            .spawn(move || {
                run_hook_message_window(
                    ready_sender,
                    control_receiver,
                    worker_state,
                    worker_lifecycle,
                )
            })
            .map_err(|error| format!("Could not start fixed-shortcut listener: {error}"))?;

        match ready_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(Ok(())) => Ok(ShortcutHookController {
                controls: control_sender,
                state,
                lifecycle,
                shutdown_requested: AtomicBool::new(false),
                worker: Mutex::new(Some(worker)),
            }),
            Ok(Err(error)) => {
                let _ = worker.join();
                lifecycle.record(LifecycleEvent::ShortcutHookFailed);
                Err(error)
            }
            Err(_) => {
                lifecycle.record(LifecycleEvent::ShortcutHookFailed);
                // Do not block application startup indefinitely if Windows
                // has not returned from the listener thread. The process will
                // abort startup; the still-detached thread has no managed app
                // state beyond the disabled acceptance gate.
                if let Some(context) = HOOK_CONTEXT.get() {
                    context.cancelled.store(true, Ordering::Release);
                    context.accepting.store(false, Ordering::Release);
                    context.advance_epoch();
                    context.reset_transient_state();
                }
                let hwnd = state.hwnd.load(Ordering::Acquire);
                if hwnd != 0 {
                    let (response, _receive) = mpsc::sync_channel(1);
                    let _ = control_sender.send(ControlRequest::Shutdown { response });
                    let _ = unsafe { PostMessageW(hwnd as HWND, CONTROL_MESSAGE, 0, 0) };
                }
                drop(worker);
                Err("Timed out while starting the fixed-shortcut listener".to_owned())
            }
        }
    }

    fn run_hook_message_window(
        ready_sender: mpsc::SyncSender<Result<(), String>>,
        controls: mpsc::Receiver<ControlRequest>,
        state: Arc<ListenerState>,
        lifecycle: LifecycleLogger,
    ) {
        let hwnd = match unsafe { create_hook_window() } {
            Ok(hwnd) => hwnd,
            Err(error) => {
                lifecycle.record(LifecycleEvent::ShortcutHookFailed);
                let _ = ready_sender.send(Err(error));
                return;
            }
        };
        state.hwnd.store(hwnd as isize, Ordering::Release);
        let context = HOOK_CONTEXT
            .get()
            .expect("Hook context must exist before start");
        if context.cancelled.load(Ordering::Acquire) {
            state.hwnd.store(0, Ordering::Release);
            unsafe { DestroyWindow(hwnd) };
            let _ = ready_sender.send(Err("The fixed-shortcut listener was cancelled".to_owned()));
            return;
        }
        context.hwnd.store(hwnd as isize, Ordering::Release);

        let mut hook = match unsafe { install_hook() } {
            Ok(hook) => hook,
            Err(error) => {
                context.hwnd.store(0, Ordering::Release);
                state.hwnd.store(0, Ordering::Release);
                unsafe { DestroyWindow(hwnd) };
                lifecycle.record(LifecycleEvent::ShortcutHookFailed);
                let _ = ready_sender.send(Err(error));
                return;
            }
        };
        if context.cancelled.load(Ordering::Acquire) {
            unsafe { UnhookWindowsHookEx(hook) };
            context.hwnd.store(0, Ordering::Release);
            state.hwnd.store(0, Ordering::Release);
            unsafe { DestroyWindow(hwnd) };
            let _ = ready_sender.send(Err("The fixed-shortcut listener was cancelled".to_owned()));
            return;
        }
        context.reset_transient_state();
        context.advance_epoch();
        // This gate is checked once in the callback and again on the message
        // window. It prevents any queued pre-pause ID from reaching storage.
        context.accepting.store(true, Ordering::Release);
        state.active.store(true, Ordering::Release);
        lifecycle.record(LifecycleEvent::ShortcutHookStarted);
        let _ = ready_sender.send(Ok(()));

        unsafe {
            let mut message: MSG = mem::zeroed();
            while GetMessageW(&mut message, ptr::null_mut(), 0, 0) > 0 {
                if message.message == CONTROL_MESSAGE {
                    if process_control_requests(&controls, &state, &lifecycle, context, &mut hook) {
                        break;
                    }
                } else {
                    DispatchMessageW(&message);
                }
            }
        }

        if !hook.is_null() {
            unsafe { UnhookWindowsHookEx(hook) };
        }
        context.accepting.store(false, Ordering::Release);
        context.reset_transient_state();
        context.hwnd.store(0, Ordering::Release);
        state.active.store(false, Ordering::Release);
        state.hwnd.store(0, Ordering::Release);
        unsafe { DestroyWindow(hwnd) };
        lifecycle.record(LifecycleEvent::ShortcutHookStopped);
    }

    unsafe fn create_hook_window() -> Result<HWND, String> {
        let instance = GetModuleHandleW(ptr::null());
        if instance.is_null() {
            return Err(last_error("Could not obtain module handle"));
        }

        let mut window_class: WNDCLASSW = mem::zeroed();
        window_class.lpfnWndProc = Some(hook_window_proc);
        window_class.hInstance = instance;
        window_class.lpszClassName = HOOK_WINDOW_CLASS.as_ptr();
        let _ = RegisterClassW(&window_class);

        let hwnd = CreateWindowExW(
            0,
            HOOK_WINDOW_CLASS.as_ptr(),
            HOOK_WINDOW_CLASS.as_ptr(),
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
            return Err(last_error("Could not create fixed-shortcut message window"));
        }
        Ok(hwnd)
    }

    unsafe fn install_hook() -> Result<HHOOK, String> {
        let instance: HINSTANCE = GetModuleHandleW(ptr::null());
        if instance.is_null() {
            return Err(last_error(
                "Could not obtain module handle for shortcut hook",
            ));
        }
        let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(low_level_keyboard_proc), instance, 0);
        if hook.is_null() {
            return Err(last_error("Could not install fixed-shortcut listener"));
        }
        Ok(hook)
    }

    fn process_control_requests(
        controls: &mpsc::Receiver<ControlRequest>,
        state: &ListenerState,
        lifecycle: &LifecycleLogger,
        context: &HookContext,
        hook: &mut HHOOK,
    ) -> bool {
        while let Ok(request) = controls.try_recv() {
            match request {
                ControlRequest::SetListening { enabled, response } => {
                    let result = if enabled {
                        if context.cancelled.load(Ordering::Acquire) {
                            Err("The fixed-shortcut listener was cancelled".to_owned())
                        } else if hook.is_null() {
                            context.accepting.store(false, Ordering::Release);
                            context.advance_epoch();
                            context.reset_transient_state();
                            unsafe {
                                drain_shortcut_messages(context.hwnd.load(Ordering::Acquire) as HWND)
                            };
                            match unsafe { install_hook() } {
                                Ok(new_hook) => {
                                    // Shutdown can race with a slow hook
                                    // installation. Recheck the cancellation
                                    // gate before this fresh handle is ever
                                    // allowed to accept an aggregate ID.
                                    if context.cancelled.load(Ordering::Acquire) {
                                        let _ = unsafe { UnhookWindowsHookEx(new_hook) };
                                        state.active.store(false, Ordering::Release);
                                        Err("The fixed-shortcut listener was cancelled".to_owned())
                                    } else {
                                        *hook = new_hook;
                                        context.accepting.store(true, Ordering::Release);
                                        state.active.store(true, Ordering::Release);
                                        lifecycle.record(LifecycleEvent::ShortcutHookStarted);
                                        Ok(())
                                    }
                                }
                                Err(error) => {
                                    lifecycle.record(LifecycleEvent::ShortcutHookFailed);
                                    Err(error)
                                }
                            }
                        } else {
                            // If a prior Unhook call reported failure, retain
                            // the conservative inactive state until an
                            // explicit resume confirms this existing handle
                            // can accept fixed aggregate IDs again.
                            context.advance_epoch();
                            context.reset_transient_state();
                            unsafe {
                                drain_shortcut_messages(context.hwnd.load(Ordering::Acquire) as HWND)
                            };
                            if context.cancelled.load(Ordering::Acquire) {
                                state.active.store(false, Ordering::Release);
                                Err("The fixed-shortcut listener was cancelled".to_owned())
                            } else {
                                context.accepting.store(true, Ordering::Release);
                                state.active.store(true, Ordering::Release);
                                lifecycle.record(LifecycleEvent::ShortcutHookStarted);
                                Ok(())
                            }
                        }
                    } else {
                        // Clear acceptance before unhooking so an already
                        // queued hook notification cannot form a stale chord.
                        context.accepting.store(false, Ordering::Release);
                        context.advance_epoch();
                        state.active.store(false, Ordering::Release);
                        context.reset_transient_state();
                        if !hook.is_null() {
                            if unsafe { UnhookWindowsHookEx(*hook) } == 0 {
                                lifecycle.record(LifecycleEvent::ShortcutHookFailed);
                                Err(last_error("Could not pause fixed-shortcut listener"))
                            } else {
                                *hook = ptr::null_mut();
                                unsafe {
                                    drain_shortcut_messages(
                                        context.hwnd.load(Ordering::Acquire) as HWND
                                    )
                                };
                                lifecycle.record(LifecycleEvent::ShortcutHookStopped);
                                Ok(())
                            }
                        } else {
                            Ok(())
                        }
                    };
                    let _ = response.send(result);
                }
                ControlRequest::Shutdown { response } => {
                    context.accepting.store(false, Ordering::Release);
                    context.advance_epoch();
                    state.active.store(false, Ordering::Release);
                    context.reset_transient_state();
                    if !hook.is_null() {
                        let _ = unsafe { UnhookWindowsHookEx(*hook) };
                        *hook = ptr::null_mut();
                    }
                    unsafe {
                        drain_shortcut_messages(context.hwnd.load(Ordering::Acquire) as HWND)
                    };
                    let _ = response.send(());
                    return true;
                }
            }
        }
        false
    }

    unsafe extern "system" fn hook_window_proc(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if message == SHORTCUT_MESSAGE {
            if let Some(context) = HOOK_CONTEXT.get() {
                if !context.cancelled.load(Ordering::Acquire)
                    && context.accepting.load(Ordering::Acquire)
                    && lparam as u64 == context.epoch.load(Ordering::Acquire)
                {
                    if let Some(shortcut_id) = ShortcutId::from_storage_id(wparam as i64) {
                        // This runs on the listener's message-pump thread, not
                        // in the low-level hook callback. Only the fixed
                        // numeric ID is passed to the aggregate collector.
                        context.collector.handle_shortcut_make(shortcut_id);
                    }
                }
            }
            return 0;
        }

        DefWindowProcW(window, message, wparam, lparam)
    }

    unsafe extern "system" fn low_level_keyboard_proc(
        code: i32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        // A low-level hook must never interfere with keyboard delivery.
        // Every branch below falls through to CallNextHookEx, including
        // negative hook codes and malformed event pointers.
        if code >= 0 && lparam != 0 {
            let keyboard = &*(lparam as *const KBDLLHOOKSTRUCT);
            let flags = keyboard.flags;
            if !is_injected(flags) {
                if let Some(context) = HOOK_CONTEXT.get() {
                    // Snapshot before inspecting the event. A pause/resume
                    // advances this generation, so a callback that began
                    // before the transition can never enqueue into the new
                    // collection session.
                    let event_epoch = context.epoch.load(Ordering::Acquire);
                    let message = wparam as u32;
                    let virtual_key = keyboard.vkCode;
                    if !context.cancelled.load(Ordering::Acquire)
                        && context.accepting.load(Ordering::Acquire)
                        && is_key_down(message)
                    {
                        if let Some(bit) = modifier_bit(virtual_key, keyboard.scanCode, flags) {
                            context.modifier_bits.fetch_or(bit, Ordering::AcqRel);
                        } else if let Some(key_id) = catalog_trigger_from_virtual_key(virtual_key) {
                            let (high, bit) = super::trigger_slot(key_id);
                            let already_held = if high {
                                context.held_trigger_high.fetch_or(bit, Ordering::AcqRel) & bit != 0
                            } else {
                                context.held_trigger_low.fetch_or(bit, Ordering::AcqRel) & bit != 0
                            };
                            if !already_held {
                                let modifier_bits = context.modifier_bits.load(Ordering::Acquire);
                                if let Some(shortcut_id) =
                                    match_catalog_shortcut(key_id, modifier_bits, flags)
                                {
                                    let hwnd = context.hwnd.load(Ordering::Acquire);
                                    if hwnd != 0 {
                                        // `wParam` contains only the fixed
                                        // numeric ID. No event pointer or raw
                                        // key data is retained or queued.
                                        let _ = PostMessageW(
                                            hwnd as HWND,
                                            SHORTCUT_MESSAGE,
                                            shortcut_id.storage_id() as usize,
                                            event_epoch as isize,
                                        );
                                    }
                                }
                            }
                        }
                    } else if !context.cancelled.load(Ordering::Acquire)
                        && context.accepting.load(Ordering::Acquire)
                        && is_key_up(message)
                    {
                        if let Some(bit) = modifier_bit(virtual_key, keyboard.scanCode, flags) {
                            context.modifier_bits.fetch_and(!bit, Ordering::AcqRel);
                        } else if let Some(key_id) = catalog_trigger_from_virtual_key(virtual_key) {
                            let (high, bit) = super::trigger_slot(key_id);
                            if high {
                                context.held_trigger_high.fetch_and(!bit, Ordering::AcqRel);
                            } else {
                                context.held_trigger_low.fetch_and(!bit, Ordering::AcqRel);
                            }
                        }
                    }
                }
            }
        }

        CallNextHookEx(ptr::null_mut(), code, wparam, lparam)
    }

    unsafe fn drain_shortcut_messages(hwnd: HWND) {
        if hwnd.is_null() {
            return;
        }
        let mut message: MSG = mem::zeroed();
        while PeekMessageW(
            &mut message,
            hwnd,
            SHORTCUT_MESSAGE,
            SHORTCUT_MESSAGE,
            PM_REMOVE,
        ) != 0
        {}
    }

    fn last_error(context: &str) -> String {
        format!("{context}: {}", std::io::Error::last_os_error())
    }
}

#[cfg(windows)]
pub use windows::{start, ShortcutHookController};

#[cfg(all(not(windows), not(target_os = "macos")))]
pub struct ShortcutHookController;

#[cfg(all(not(windows), not(target_os = "macos")))]
impl ShortcutHookController {
    pub fn is_active(&self) -> bool {
        false
    }

    pub fn set_listening(&self, _enabled: bool) -> Result<(), String> {
        Err("Keyboard Heatmap V1 supports Windows keyboard shortcuts only".to_owned())
    }

    pub fn shutdown(&self) {}
}

#[cfg(all(not(windows), not(target_os = "macos")))]
pub fn start(
    _collector: std::sync::Arc<crate::collector::KeyCollector>,
    _lifecycle: crate::lifecycle::LifecycleLogger,
) -> Result<ShortcutHookController, String> {
    Err("Keyboard Heatmap V1 supports Windows keyboard shortcuts only".to_owned())
}

/// macOS has no separate listener: the HID event tap owned by `raw_input` also
/// matches the fixed shortcut catalogue, so this control only reports the
/// combined listener as active.
#[cfg(target_os = "macos")]
pub struct ShortcutHookController;

#[cfg(target_os = "macos")]
impl ShortcutHookController {
    pub fn is_active(&self) -> bool {
        true
    }

    pub fn set_listening(&self, _enabled: bool) -> Result<(), String> {
        Ok(())
    }

    pub fn shutdown(&self) {}
}

#[cfg(target_os = "macos")]
pub fn start(
    _collector: std::sync::Arc<crate::collector::KeyCollector>,
    _lifecycle: crate::lifecycle::LifecycleLogger,
) -> Result<ShortcutHookController, String> {
    Ok(ShortcutHookController)
}

#[cfg(test)]
mod tests {
    use super::{
        HookMatcher, LLKHF_INJECTED_FLAG, WM_KEYDOWN_MESSAGE, WM_KEYUP_MESSAGE,
        WM_SYSKEYDOWN_MESSAGE,
    };
    use crate::shortcuts::ShortcutId;

    #[test]
    fn recognizes_windows_and_alt_system_shortcuts_from_fixed_events() {
        let mut matcher = HookMatcher::default();
        assert_eq!(matcher.observe(WM_KEYDOWN_MESSAGE, 0x5B, 0), None);
        assert_eq!(
            matcher.observe(WM_KEYDOWN_MESSAGE, 0x44, 0),
            Some(ShortcutId::WinD)
        );

        let mut matcher = HookMatcher::default();
        assert_eq!(matcher.observe(WM_SYSKEYDOWN_MESSAGE, 0xA4, 0), None);
        assert_eq!(
            matcher.observe(WM_SYSKEYDOWN_MESSAGE, 0x09, 0x20),
            Some(ShortcutId::AltTab)
        );

        let mut matcher = HookMatcher::default();
        assert_eq!(matcher.observe(WM_KEYDOWN_MESSAGE, 0xA0, 0), None);
        assert_eq!(matcher.observe(WM_SYSKEYDOWN_MESSAGE, 0xA4, 0), None);
        assert_eq!(
            matcher.observe(WM_SYSKEYDOWN_MESSAGE, 0x09, 0x20),
            Some(ShortcutId::AltShiftTab)
        );

        let mut matcher = HookMatcher::default();
        assert_eq!(matcher.observe(WM_SYSKEYDOWN_MESSAGE, 0xA4, 0), None);
        assert_eq!(
            matcher.observe(WM_SYSKEYDOWN_MESSAGE, 0x73, 0x20),
            Some(ShortcutId::AltF4)
        );

        let mut matcher = HookMatcher::default();
        assert_eq!(matcher.observe(WM_KEYDOWN_MESSAGE, 0xA2, 0), None);
        assert_eq!(
            matcher.observe(WM_KEYDOWN_MESSAGE, 0x43, 0),
            Some(ShortcutId::CtrlC)
        );
    }

    #[test]
    fn shortcut_trigger_repeat_and_wrong_order_never_duplicate_or_backfill() {
        let mut matcher = HookMatcher::default();
        assert_eq!(matcher.observe(WM_KEYDOWN_MESSAGE, 0x44, 0), None);
        assert_eq!(matcher.observe(WM_KEYDOWN_MESSAGE, 0x5B, 0), None);
        // D is still down, so enabling Win afterwards cannot backfill a chord
        // through its repeat messages.
        assert_eq!(matcher.observe(WM_KEYDOWN_MESSAGE, 0x44, 0), None);
        assert_eq!(matcher.observe(WM_KEYUP_MESSAGE, 0x44, 0), None);
        assert_eq!(
            matcher.observe(WM_KEYDOWN_MESSAGE, 0x44, 0),
            Some(ShortcutId::WinD)
        );
        assert_eq!(matcher.observe(WM_KEYDOWN_MESSAGE, 0x44, 0), None);
    }

    #[test]
    fn fixed_catalog_rejects_extra_and_unknown_combinations() {
        let mut matcher = HookMatcher::default();
        assert_eq!(matcher.observe(WM_KEYDOWN_MESSAGE, 0x5B, 0), None);
        assert_eq!(matcher.observe(WM_SYSKEYDOWN_MESSAGE, 0xA4, 0), None);
        assert_eq!(matcher.observe(WM_KEYDOWN_MESSAGE, 0x44, 0), None);

        let mut matcher = HookMatcher::default();
        assert_eq!(matcher.observe(WM_KEYDOWN_MESSAGE, 0x5B, 0), None);
        assert_eq!(matcher.observe(WM_KEYDOWN_MESSAGE, 0x51, 0), None);
    }

    #[test]
    fn injected_events_do_not_change_transient_matcher_state() {
        let mut matcher = HookMatcher::default();
        assert_eq!(
            matcher.observe(WM_KEYDOWN_MESSAGE, 0x5B, LLKHF_INJECTED_FLAG),
            None
        );
        assert_eq!(matcher.observe(WM_KEYDOWN_MESSAGE, 0x44, 0), None);
    }

    #[test]
    fn releasing_one_modifier_side_keeps_the_other_side_active() {
        let mut matcher = HookMatcher::default();
        assert_eq!(matcher.observe(WM_SYSKEYDOWN_MESSAGE, 0xA4, 0), None);
        assert_eq!(matcher.observe(WM_SYSKEYDOWN_MESSAGE, 0xA5, 0), None);
        assert_eq!(matcher.observe(WM_KEYUP_MESSAGE, 0xA4, 0), None);
        assert_eq!(
            matcher.observe(WM_SYSKEYDOWN_MESSAGE, 0x09, 0x20),
            Some(ShortcutId::AltTab)
        );
    }

    #[test]
    fn generic_modifier_virtual_keys_preserve_sides_from_scan_code() {
        let mut matcher = HookMatcher::default();
        // Both physical Ctrl keys can be delivered as generic VK_CONTROL. The
        // E0 flag distinguishes the right-side key, so releasing the left one
        // must not erase the right-side Ctrl state. This uses no Alt state
        // flag, proving the retained side-aware modifier state itself works.
        assert_eq!(
            matcher.observe_with_scan(WM_KEYDOWN_MESSAGE, 0x11, 0x1D, 0),
            None
        );
        assert_eq!(
            matcher.observe_with_scan(WM_KEYDOWN_MESSAGE, 0x11, 0x1D, 0x0001),
            None
        );
        assert_eq!(
            matcher.observe_with_scan(WM_KEYUP_MESSAGE, 0x11, 0x1D, 0),
            None
        );
        assert_eq!(
            matcher.observe(WM_KEYDOWN_MESSAGE, 0x43, 0),
            Some(ShortcutId::CtrlC)
        );

        let mut matcher = HookMatcher::default();
        assert_eq!(
            matcher.observe_with_scan(WM_KEYDOWN_MESSAGE, 0x11, 0x1D, 0),
            None
        );
        assert_eq!(
            matcher.observe_with_scan(WM_KEYDOWN_MESSAGE, 0x10, 0x2A, 0),
            None
        );
        assert_eq!(
            matcher.observe_with_scan(WM_KEYDOWN_MESSAGE, 0x10, 0x36, 0),
            None
        );
        assert_eq!(
            matcher.observe_with_scan(WM_KEYUP_MESSAGE, 0x10, 0x2A, 0),
            None
        );
        assert_eq!(
            matcher.observe(WM_KEYDOWN_MESSAGE, 0x5A, 0),
            Some(ShortcutId::CtrlShiftZ)
        );
    }
}
