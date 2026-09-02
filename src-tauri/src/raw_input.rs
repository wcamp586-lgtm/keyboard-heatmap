//! Windows Raw Input receiver.
//!
//! It registers a message-only window for the keyboard usage page with
//! `RIDEV_INPUTSINK`, reads individual `WM_INPUT` messages, maps them
//! immediately to a fixed `KeyId`, then discards the raw payload. It never
//! calls text/layout APIs, never blocks input, and never alters other apps'
//! keyboard handling.

#[cfg(not(windows))]
use std::sync::Arc;

#[cfg(not(windows))]
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

#[cfg(not(windows))]
pub struct RawInputController;

#[cfg(not(windows))]
impl RawInputController {
    pub fn is_active(&self) -> bool {
        false
    }

    pub fn set_listening(&self, _enabled: bool) -> Result<(), String> {
        Err("Keyboard Heatmap V1 supports Windows Raw Input only".to_owned())
    }
}

#[cfg(not(windows))]
pub fn start(
    _collector: Arc<KeyCollector>,
    _lifecycle: crate::lifecycle::LifecycleLogger,
) -> Result<RawInputController, String> {
    Err("Keyboard Heatmap V1 supports Windows Raw Input only".to_owned())
}
