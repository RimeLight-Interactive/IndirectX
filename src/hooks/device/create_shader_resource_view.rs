use crate::fn_typedefs::device::CreateShaderResourceView;
use std::sync::OnceLock;
use std::ffi::c_void;
use windows::Win32::Graphics::Direct3D11::*;
use windows_result::HRESULT;

static ORIG_FUNC: OnceLock<CreateShaderResourceView> = OnceLock::new();

pub fn set_orig_func(func: usize) {
    let _ = ORIG_FUNC.set(unsafe { std::mem::transmute(func) });
}

pub fn hooked_func(
    this: *mut c_void,
    resource: *mut ID3D11Resource,
    shader_resource_view_desc: *const D3D11_SHADER_RESOURCE_VIEW_DESC,
    shaderresourceview: *mut *mut ID3D11ShaderResourceView,
) -> HRESULT {
    unsafe {
        let func = ORIG_FUNC.get().unwrap();
        func(this, resource, shader_resource_view_desc, shaderresourceview)
    }
}
