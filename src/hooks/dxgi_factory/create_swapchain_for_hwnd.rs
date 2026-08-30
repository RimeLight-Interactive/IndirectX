use crate::fn_typedefs::dxgi_factory::CreateSwapChainForHwnd;
use std::ffi::c_void;
use std::sync::OnceLock;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Graphics::Dxgi::{
    DXGI_SWAP_CHAIN_DESC1,
    DXGI_SWAP_CHAIN_FULLSCREEN_DESC,
};
use windows_result::HRESULT;

static ORIG_FUNC: OnceLock<CreateSwapChainForHwnd> = OnceLock::new();

pub fn set_orig_func(func: usize) {
    let _ = ORIG_FUNC.set(unsafe { std::mem::transmute(func) });
}

pub fn hooked_func(
    this: *mut c_void,
    device: *mut c_void,
    hwnd: HWND,
    desc: *const DXGI_SWAP_CHAIN_DESC1,
    fullscreen_desc: *const DXGI_SWAP_CHAIN_FULLSCREEN_DESC,
    restrict_to_output: *mut c_void,
    pp_swapchain: *mut *mut c_void,
) -> HRESULT {
    unsafe {
        let func = ORIG_FUNC.get().unwrap();

        let result = func(
            this,
            device,
            hwnd,
            desc,
            fullscreen_desc,
            restrict_to_output,
            pp_swapchain,
        );
        if !result.is_ok() || pp_swapchain.is_null() || (*pp_swapchain).is_null() {
            return result;
        }
        super::super::swapchain::install_swapchain_hooks(*pp_swapchain);

        result
    }
}