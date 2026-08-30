use crate::fn_typedefs::dxgi_factory::CreateSwapChainForCoreWindow;
use std::ffi::c_void;
use std::sync::OnceLock;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Graphics::Dxgi::DXGI_SWAP_CHAIN_DESC1;
use windows_result::HRESULT;

static ORIG_FUNC: OnceLock<CreateSwapChainForCoreWindow> = OnceLock::new();

pub fn set_orig_func(func: usize) {
    let _ = ORIG_FUNC.set(unsafe { std::mem::transmute(func) });
}

pub fn hooked_func(
    this: *mut c_void,
    device: *mut c_void,
    window: *mut c_void,
    desc: *const DXGI_SWAP_CHAIN_DESC1,
    restrict_to_output: *mut c_void,
    pp_swapchain: *mut *mut c_void,
) -> HRESULT {
    unsafe {
        let func = ORIG_FUNC.get().unwrap();

        let result = func(
            this,
            device,
            window,
            desc,
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