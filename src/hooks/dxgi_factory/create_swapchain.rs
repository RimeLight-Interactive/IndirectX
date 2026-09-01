use crate::fn_typedefs::dxgi_factory::CreateSwapChain;
use crate::log;
use std::sync::OnceLock;
use std::ffi::c_void;
use windows::Win32::Graphics::Dxgi::{Common::*, DXGI_SWAP_CHAIN_DESC};
use windows_result::HRESULT;

static ORIG_FUNC: OnceLock<CreateSwapChain> = OnceLock::new();

pub fn set_orig_func(func: usize) {
    let _ = ORIG_FUNC.set(unsafe { std::mem::transmute(func) });
}

pub fn hooked_func(
    this: *mut c_void,
    device: *mut c_void,
    desc: *const DXGI_SWAP_CHAIN_DESC,
    pp_swapchain: *mut *mut c_void,
) -> HRESULT {
    unsafe {
        log!("Intercepted swapchain creation. Normal");
        let func = ORIG_FUNC.get().unwrap();
        let result = func(this, device, desc, pp_swapchain);
        if !result.is_ok() || pp_swapchain.is_null() || (*pp_swapchain).is_null() {
            return result;
        }
        super::super::swapchain::install_swapchain_hooks(*pp_swapchain);
        log!("installed swapchain hooks");
        result
    }
}
