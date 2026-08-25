use crate::{fn_typedefs::context::Dispatch, log};
use std::sync::OnceLock;
use std::ffi::c_void;
use windows::Win32::Graphics::{ Direct3D11::*, Dxgi::Common::DXGI_FORMAT};
use windows_result::HRESULT;
use windows::core::BOOL;
use crate::special_ops::{
    shader_manager::{is_active_cs_allowed, get_current_cs_hash},
    cbv_patch_manager
};

static ORIG_FUNC: OnceLock<Dispatch> = OnceLock::new();

pub fn set_orig_func(func: usize) {
    let _ = ORIG_FUNC.set(unsafe { std::mem::transmute(func) });
}

pub fn hooked_func(
    this: *mut c_void,
    a: u32,
    b: u32,
    c: u32,
) {
    unsafe {
        let hash = get_current_cs_hash(this as usize).unwrap_or(0);
        if !is_active_cs_allowed(hash){
            return;
        }
        if let Some(cbv_patch) = cbv_patch_manager::get_patches(hash){
            cbv_patch_manager::apply_patches_cs(this, &cbv_patch);
        }
        let func = ORIG_FUNC.get().unwrap();
        func(this, a, b, c)
    }
}
