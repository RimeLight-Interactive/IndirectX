use crate::fn_typedefs::device::CreateDeferredContext;
use std::sync::OnceLock;
use std::ffi::c_void;
use windows::Win32::Graphics::Direct3D11::*;
use windows_result::HRESULT;

static ORIG_FUNC: OnceLock<CreateDeferredContext> = OnceLock::new();

pub fn set_orig_func(func: usize) {
    let _ = ORIG_FUNC.set(unsafe { std::mem::transmute(func) });
}

pub fn hooked_func(
    this: *mut c_void,
    a: u32,
    devicecontext: *mut *mut ID3D11DeviceContext,
) -> HRESULT {
    unsafe {
        let func = ORIG_FUNC.get().unwrap();
        func(this, a, devicecontext)
    }
}
