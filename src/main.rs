#![feature(str_from_raw_parts)]
#![no_main]
#![no_std]

// Required for panic handler
extern crate flipperzero_rt;

// Required for allocator
extern crate alloc;
extern crate flipperzero_alloc;

use alloc::sync::Arc;
use core::convert::Into;
use core::ffi::CStr;
use flipperzero_sys as sys;

use flipperzero::dialogs::DialogMessageButton;
use flipperzero::{
    dialogs, format,
    furi::{sync::Mutex, thread, time::Instant},
    gui::canvas::Align,
    info,
};

// Define the FAP Manifest for this application
flipperzero_rt::manifest!(
    name = "Flipper auto click",
    app_version = 1,
    has_icon = true,
    // See https://github.com/flipperzero-rs/flipperzero/blob/v0.11.0/docs/icons.md for icon format
    icon = "rustacean-10x10.icon",
);

// Define the entry function
flipperzero_rt::entry!(main);

// Entry point
fn main(_args: Option<&CStr>) -> i32 {
    let origin;
    unsafe {
        sys::furi_hal_usb_unlock();
        origin = sys::furi_hal_usb_get_config();
        if !sys::furi_hal_usb_set_config(
            core::ptr::addr_of_mut!(sys::usb_hid),
            core::ptr::null_mut(),
        ) {
            info!("Can't set `usb_hid` as device");
            dialogs::alert("Can't set `usb_hid` as device");
            return 1;
        } else {
            info!("The `usb_hid` device is configured!");
            dialogs::alert("The `usb_hid` device is configured!");
        }
    }

    let cfg = Arc::new(Mutex::new(CFG {
        enabled: false,
        halt: false,
        frequency: INIT,
    }));

    let _shared_cfg = cfg.clone();
    let handle = thread::spawn(move || {
        let cfg = _shared_cfg;

        let (mut enabled, mut halt, mut frequency) = {
            let cfg = cfg.lock();
            (cfg.enabled, cfg.halt, cfg.frequency)
        };

        loop {
            thread::sleep(core::time::Duration::from_millis(frequency));

            (enabled, halt, frequency) = {
                let cfg = cfg.lock();
                (cfg.enabled, cfg.halt, cfg.frequency)
            };

            if halt {
                return 0;
            }

            if enabled {
                unsafe {
                    press(HidMouseButtons::HID_MOUSE_BTN_LEFT, HOLD);
                }
            }
        }
    });

    let mut dialogs = dialogs::DialogsApp::open();
    let mut body;
    let (mut frequency, mut enabled) = {
        let cfg = cfg.lock();
        (cfg.frequency, cfg.enabled)
    };

    loop {
        let mut message = dialogs::DialogMessage::new();
        message.set_header(c"Auto clicker", 0, 0, Align::Left, Align::Top);
        body = format!(
            "Frequency: {}ms\nActive: {}",
            frequency,
            if enabled { "yes" } else { "no" },
        );
        message.set_text(body.as_c_str(), 0, 10, Align::Left, Align::Top);
        message.set_buttons(
            Some(c"less"),
            Some(if enabled { c"disable" } else { c"enable" }),
            Some(c"more"),
        );

        match dialogs.show_message(&message) {
            DialogMessageButton::Back => {
                info!("Stopping...");
                break;
            }
            DialogMessageButton::Left => {
                info!("Down frequency");
                if frequency > 0 {
                    frequency -= STEP;
                }
            }
            DialogMessageButton::Right => {
                info!("Up frequency");
                frequency += STEP;
            }
            DialogMessageButton::Center => {
                if enabled {
                    info!("Disable")
                } else {
                    info!("Enable")
                }
                enabled = !enabled;
            }
        }

        {
            let mut cfg = cfg.lock();
            cfg.frequency = frequency;
            cfg.enabled = enabled;
        }
    }

    {
        let mut cfg = cfg.lock();
        cfg.halt = true;
    }
    info!("Halt");
    let status = handle.join();
    info!("Clicker exit status {:?}", status);

    unsafe { !sys::furi_hal_usb_set_config(origin, core::ptr::null_mut()) as _ }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, ufmt::derive::uDebug)]
#[allow(dead_code, non_camel_case_types)]
enum HidMouseButtons {
    HID_MOUSE_BTN_LEFT = (1 << 0),
    HID_MOUSE_BTN_RIGHT = (1 << 1),
    HID_MOUSE_BTN_WHEEL = (1 << 2),
}

unsafe fn press(btn: HidMouseButtons, ms: u32) {
    info!("Pressing button {:?} {}ms", btn, ms);
    sys::furi_hal_hid_mouse_press(btn as u8);
    for key in KEYS {
        sys::furi_hal_hid_kb_press(*key);
    }
    sys::furi_delay_us(ms * 1000); // micro hitch
    sys::furi_hal_hid_mouse_release(btn as u8);
    for key in KEYS {
        sys::furi_hal_hid_kb_release(*key);
    }
}

fn ptr_to_str(ptr: *mut core::ffi::c_void) -> &'static str {
    if ptr.is_null() {
        "NULL"
    } else {
        unsafe { core::str::from_raw_parts(ptr as _, sys::strlen(ptr as _) as _) }
    }
}

struct CFG {
    enabled: bool,
    halt: bool,
    frequency: u64,
}

const STEP: u64 = 20;
const INIT: u64 = 100;
const HOLD: u32 = 30;
const KEYS: &'static [u16] = &[
    // 0 - 9
    0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39,
    // A - O
    0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x4F,
    // P - Z
    0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A,
];
