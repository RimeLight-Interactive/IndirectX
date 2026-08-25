use serde::de;
use xxhash_rust;
use std::ffi::c_void;
use crate::hooks::device;
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK,  MB_SYSTEMMODAL, MB_TASKMODAL};
use windows::core::PCWSTR;
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Diagnostics::Debug::OutputDebugStringW;

pub fn hash(pointer: *const c_void, len: usize) -> u64{
    let bytecode = unsafe {std::slice::from_raw_parts(pointer as *const u8, len)};
    xxhash_rust::xxh3::xxh3_64(bytecode)
}

pub fn get_device() -> Option<*mut c_void> {
    unsafe {
        device::DEVICE
    }
}

pub fn show_fatal_error(title: &str, message: &str) {
    let full_log = format!("[FATAL ERROR] {title}:\n{message}\n");

    // 1. Always dump directly to a dedicated text file on disk
    let _ = std::fs::write("INDIRECTX_FATAL_ERROR.txt", &full_log);

    // 2. Output to Windows Debugger (viewable in DebugView or Visual Studio)
    let debug_wide: Vec<u16> = full_log.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        OutputDebugStringW(PCWSTR(debug_wide.as_ptr()));
    }

    // 3. Attempt native MessageBox (may fail inside DllMain/Loader Lock)
    let title_wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    let msg_wide: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        MessageBoxW(
            Some(HWND(std::ptr::null_mut())),
            PCWSTR(msg_wide.as_ptr()),
            PCWSTR(title_wide.as_ptr()),
            MB_OK | MB_ICONERROR | MB_TASKMODAL | MB_SYSTEMMODAL,
        );
    }
}