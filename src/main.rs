#![feature(str_from_raw_parts)]
#![no_main]
#![no_std]

// Required for panic handler
extern crate flipperzero_rt;

// Required for allocator
extern crate alloc;
extern crate flipperzero_alloc;

use core::ffi::CStr;
use flipperzero_sys as sys;

use flipperzero::{dialogs, format, gui::canvas::Align, info};

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
    // To customize the dialog, use the DialogMessage API:
    let mut dialogs = dialogs::DialogsApp::open();
    let mut frequency: u32 = 200;
    let mut active = false;
    let mut msg = format!(
        "Frequency: {}ms\nActive: {}",
        frequency,
        if active { "yes" } else { "no" },
    );
    let mut origin;

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

    loop {
        {
            let mut message = dialogs::DialogMessage::new();
            message.set_header(c"Auto clicker", 0, 0, Align::Left, Align::Top);
            message.set_text(msg.as_c_str(), 0, 10, Align::Left, Align::Top);
            message.set_buttons(
                Some(c"less"),
                Some(if active { c"no" } else { c"yes" }),
                Some(c"more"),
            );

            match dialogs.show_message(&message) {
                dialogs::DialogMessageButton::Back => {
                    info!("Exit...");
                    break;
                }
                dialogs::DialogMessageButton::Left if frequency > 0 => {
                    info!("Down frequency");
                    frequency -= 10;
                }
                dialogs::DialogMessageButton::Right => {
                    info!("Up frequency");
                    frequency += 10;
                }
                dialogs::DialogMessageButton::Center => {
                    info!("Switch activity");
                    active = !active
                }
                _ => {}
            }
        }

        unsafe {
            press(HidMouseButtons::HID_MOUSE_BTN_LEFT, frequency);
            press(HidMouseButtons::HID_MOUSE_BTN_LEFT, frequency);
            press(HidMouseButtons::HID_MOUSE_BTN_LEFT, frequency);
        }

        msg = format!(
            "Frequency: {}ms\nActive: {}",
            frequency,
            if active { "yes" } else { "no" },
        );
    }

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
    let press = sys::furi_hal_hid_mouse_press(btn as u8);
    sys::furi_delay_us(ms * 1000); // micro hitch
    let release = sys::furi_hal_hid_mouse_release(btn as u8);
    sys::furi_delay_us(ms * 1000);
    info!("Result press={} release={}", press, release);
    if !press || !release {
        let msg = format!("Result press={} release={}", press, release);
        dialogs::alert(ptr_to_str(msg.as_c_ptr() as _));
    }
}

fn ptr_to_str(ptr: *mut core::ffi::c_void) -> &'static str {
    if ptr.is_null() {
        "NULL"
    } else {
        unsafe { core::str::from_raw_parts(ptr as _, sys::strlen(ptr as _) as _) }
    }
}
