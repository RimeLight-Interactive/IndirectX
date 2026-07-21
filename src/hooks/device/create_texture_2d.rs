use crate::fn_typedefs::device::CreateTexture2D;
use std::sync::OnceLock;
use std::ffi::c_void;
use windows::Win32::Graphics::Direct3D11::*;
use windows_result::HRESULT;

static ORIG_FUNC: OnceLock<CreateTexture2D> = OnceLock::new();

pub fn set_orig_func(func: usize) {
    let _ = ORIG_FUNC.set(unsafe { std::mem::transmute(func) });
}

pub fn hooked_func(
    this: *mut c_void,
    texture2d_desc: *const D3D11_TEXTURE2D_DESC,
    subresource_data: *const D3D11_SUBRESOURCE_DATA,
    texture2d: *mut *mut ID3D11Texture2D,
) -> HRESULT {
    unsafe {
        let func = ORIG_FUNC.get().unwrap();
        func(this, texture2d_desc, subresource_data, texture2d)
    }
}
