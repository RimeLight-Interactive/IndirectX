use crate::fn_typedefs::dxgi_factory::QueryInterface;
use crate::log;
use std::sync::OnceLock;
use std::ffi::c_void;
use windows::{Win32::Graphics::Dxgi::{Common::*, DXGI_SWAP_CHAIN_DESC}, core::GUID};
use windows_result::HRESULT;
use super::{install_dxgi_factory_hooks, install_dxgi_factory2_hooks};

static ORIG_FUNC: OnceLock<QueryInterface> = OnceLock::new();

pub fn set_orig_func(func: usize) {
    let _ = ORIG_FUNC.set(unsafe { std::mem::transmute(func) });
}

pub fn hooked_func(
    this: *mut c_void,
    refiid: *const GUID,
    pp_new_interface: *mut *mut c_void,
) -> HRESULT {
    unsafe {
        log!("Intercepted factory QueryInterface.");
        log!("REFIID: {:?}", *refiid);

        let func = ORIG_FUNC.get().unwrap();
        let result = func(this, refiid, pp_new_interface);
        result
    }
}
