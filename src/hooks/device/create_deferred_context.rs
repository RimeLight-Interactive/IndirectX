use crate::fn_typedefs::device::CreateDeferredContext;
use crate::log;
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
        let res = func(this, a, devicecontext);
        log!("result: {}", res);
        //super::super::context::install_context_hooks(*devicecontext as *mut c_void);
        res
    }
}
