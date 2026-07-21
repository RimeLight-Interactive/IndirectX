use crate::fn_typedefs::context::ResolveSubresource;
use std::sync::OnceLock;
use std::ffi::c_void;
use windows::Win32::Graphics::{ Direct3D11::*, Dxgi::Common::DXGI_FORMAT};
use windows_result::HRESULT;
use windows::core::BOOL;

static ORIG_FUNC: OnceLock<ResolveSubresource> = OnceLock::new();

pub fn set_orig_func(func: usize) {
    let _ = ORIG_FUNC.set(unsafe { std::mem::transmute(func) });
}

pub fn hooked_func(
    this: *mut c_void,
    resource: *mut ID3D11Resource,
    a: u32,
    resource_1: *mut ID3D11Resource,
    b: u32,
    c: DXGI_FORMAT,
) {
    unsafe {
        let func = ORIG_FUNC.get().unwrap();
        func(this, resource, a, resource_1, b, c)
    }
}
