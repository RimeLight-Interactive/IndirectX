use crate::fn_typedefs::context::OMSetRenderTargetsAndUnorderedAccessViews;
use std::sync::OnceLock;
use std::ffi::c_void;
use windows::Win32::Graphics::{ Direct3D11::*, Dxgi::Common::DXGI_FORMAT};
use windows_result::HRESULT;
use windows::core::BOOL;

static ORIG_FUNC: OnceLock<OMSetRenderTargetsAndUnorderedAccessViews> = OnceLock::new();

pub fn set_orig_func(func: usize) {
    let _ = ORIG_FUNC.set(unsafe { std::mem::transmute(func) });
}

pub fn hooked_func(
    this: *mut c_void,
    a: u32,
    rendertargetview: *const *mut ID3D11RenderTargetView,
    depthstencilview: *mut ID3D11DepthStencilView,
    b: u32,
    c: u32,
    unorderedaccessview: *const *mut ID3D11UnorderedAccessView,
    d: *const u32,
) {
    unsafe {
        let func = ORIG_FUNC.get().unwrap();
        func(this, a, rendertargetview, depthstencilview, b, c, unorderedaccessview, d)
    }
}
