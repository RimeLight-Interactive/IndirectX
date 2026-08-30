use crate::fn_typedefs::swapchain::Present;
use crate::log;
use std::sync::OnceLock;
use std::ffi::c_void;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows_result::HRESULT;

static ORIG_FUNC: OnceLock<Present> = OnceLock::new();

pub fn set_orig_func(func: usize) {
    let _ = ORIG_FUNC.set(unsafe { std::mem::transmute(func) });
}

pub fn hooked_func(
    this: *mut c_void,
    a: u32,
    b: u32,
) -> HRESULT {
    unsafe {
        log!("Present!!!");
        let func = ORIG_FUNC.get().unwrap();
        func(this, a, b)
    }
}
