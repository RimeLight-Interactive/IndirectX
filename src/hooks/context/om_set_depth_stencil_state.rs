use crate::fn_typedefs::context::OMSetDepthStencilState;
use crate::helpers::get_device;
use crate::log;
use std::ffi::c_void;
use std::sync::OnceLock;
use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Direct3D::{
    D3D_PRIMITIVE_TOPOLOGY, D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
    Fxc::{D3DCompile, D3DCOMPILE_OPTIMIZATION_LEVEL0},
    ID3DBlob,
};
use windows::Win32::Graphics::Direct3D11::*;
use windows::core::{Interface, PCSTR};

static ORIG_FUNC: OnceLock<OMSetDepthStencilState> = OnceLock::new();

pub fn set_orig_func(func: usize) {
    let _ = ORIG_FUNC.set(unsafe { std::mem::transmute(func) });
}

pub fn hooked_func(
    this: *mut c_void,
    depthstencilstate: *mut ID3D11DepthStencilState,
    stencil_ref: u32,
) {
    log!("sanity check");
    unsafe {
        let func = ORIG_FUNC.get().unwrap();

        if false {

            let mut desc = D3D11_DEPTH_STENCIL_DESC::default();
            let state = ID3D11DepthStencilState::from_raw(depthstencilstate as *mut c_void);
            
            state.GetDesc(&mut desc);
            log!(
                "[OMSetDSS] DepthEnable={} WriteMask={} DepthFunc={} StencilEnable={}",
                desc.DepthEnable.as_bool(),
                desc.DepthWriteMask.0,
                desc.DepthFunc.0,
                desc.StencilEnable.as_bool(),
            );
        }

        func(this, depthstencilstate, stencil_ref);
    }
}