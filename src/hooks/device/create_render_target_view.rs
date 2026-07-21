use crate::fn_typedefs::device::CreateRenderTargetView;
use std::sync::OnceLock;
use std::ffi::c_void;
use windows::Win32::Graphics::Direct3D11::*;
use windows_result::HRESULT;

static ORIG_FUNC: OnceLock<CreateRenderTargetView> = OnceLock::new();

pub fn set_orig_func(func: usize) {
    let _ = ORIG_FUNC.set(unsafe { std::mem::transmute(func) });
}

pub fn hooked_func(
    this: *mut c_void,
    resource: *mut ID3D11Resource,
    render_target_view_desc: *const D3D11_RENDER_TARGET_VIEW_DESC,
    rendertargetview: *mut *mut ID3D11RenderTargetView,
) -> HRESULT {
    unsafe {
        let func = ORIG_FUNC.get().unwrap();
        func(this, resource, render_target_view_desc, rendertargetview)
    }
}
