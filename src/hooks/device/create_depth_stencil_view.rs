use crate::fn_typedefs::device::CreateDepthStencilView;
use crate::log;
use std::sync::OnceLock;
use std::ffi::c_void;
use axum::extract::FromRef;
use dxbc_ir_parser::build::log;
use windows::Win32::Graphics::Direct3D11::*;
use windows::core::Interface;
use windows_result::HRESULT;

static ORIG_FUNC: OnceLock<CreateDepthStencilView> = OnceLock::new();

pub fn set_orig_func(func: usize) {
    let _ = ORIG_FUNC.set(unsafe { std::mem::transmute(func) });
}

pub unsafe extern "system" fn hooked_func(
    this: *mut c_void,
    resource: *mut ID3D11Resource,
    depth_stencil_view_desc: *const D3D11_DEPTH_STENCIL_VIEW_DESC,
    depthstencilview: *mut *mut ID3D11DepthStencilView,
) -> HRESULT {
    unsafe {
        let func = ORIG_FUNC.get().unwrap();

        if !resource.is_null() && !depth_stencil_view_desc.is_null() {
            log!("createDSV called with flags: {}, format.0 :{}, ViewDimension.0 :{} ", (*depth_stencil_view_desc).Flags,
            (*depth_stencil_view_desc).Format.0, (*depth_stencil_view_desc).ViewDimension.0);
            if (*depth_stencil_view_desc).ViewDimension.0 == 3 {
                log!("Casting to resource from raw pointer");
                let raw_resource = resource as *mut c_void;
                let ref_ref_resource = ID3D11Resource::from_raw_borrowed(&raw_resource);
                let Some(ref_resource) = ref_ref_resource else {
                    log!("Failed borrowed resource creation");
                    return func(this, resource , depth_stencil_view_desc, depthstencilview);
                };
                log!("DSV resource is ID3D11Texture2D");
                let ref_texture = ref_resource.cast::<ID3D11Texture2D>();
                let Ok(texture) = ref_texture else {
                    log!("failed cast");
                    return func(this, resource, depth_stencil_view_desc, depthstencilview);
                }; 
                let mut texture_desc = D3D11_TEXTURE2D_DESC::default();
                texture.GetDesc(&mut texture_desc);
                log!("GetDesc completed");
                log!("Texture dimensions: {}x{}", texture_desc.Width, texture_desc.Height);
                log!("Texture format: {}", texture_desc.Format.0);
            }
        }
        func(this, resource, depth_stencil_view_desc, depthstencilview)
    }
}
