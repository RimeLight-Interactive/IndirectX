use crate::fn_typedefs::context::Draw;
use std::sync::OnceLock;
use std::ffi::c_void;
use windows::Win32::Graphics::{ Direct3D11::*, Dxgi::Common::DXGI_FORMAT};
use windows_result::HRESULT;
use windows::core::BOOL;
use crate::special_ops::shader_manager::{is_active_ps_allowed, get_current_ps_hash};

static ORIG_FUNC: OnceLock<Draw> = OnceLock::new();

pub fn set_orig_func(func: usize) {
    let _ = ORIG_FUNC.set(unsafe { std::mem::transmute(func) });
}

pub fn hooked_func(
    this: *mut c_void,
    a: u32,
    b: u32,
) {
    unsafe {
        if !is_active_ps_allowed(get_current_ps_hash(this as usize).unwrap_or(0)){
            return;
        }
        let func = ORIG_FUNC.get().unwrap();
        func(this, a, b)
    }
}
