use crate::fn_typedefs::context::DrawIndexedInstanced;
use std::sync::OnceLock;
use std::ffi::c_void;
use windows::Win32::Graphics::{ Direct3D11::*, Dxgi::Common::DXGI_FORMAT};
use windows_result::HRESULT;
use windows::core::BOOL;

static ORIG_FUNC: OnceLock<DrawIndexedInstanced> = OnceLock::new();

pub fn set_orig_func(func: usize) {
    let _ = ORIG_FUNC.set(unsafe { std::mem::transmute(func) });
}

pub fn hooked_func(
    this: *mut c_void,
    a: u32,
    b: u32,
    c: u32,
    d: i32,
    e: u32,
) {
    unsafe {
        let func = ORIG_FUNC.get().unwrap();
        func(this, a, b, c, d, e)
    }
}
