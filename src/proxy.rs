// Proxy functions for D3D11 exports
use crate::log;
//use crate::hooks::device::install_device_hooks;
use std::ffi::c_void;
use vmt_hook::VTableHook;

type D3D11CreateDevice_fn = unsafe extern "system" fn(
    pAdapter: *mut c_void,
    DriverType: u32,
    Software: *mut c_void,
    Flags: u32,
    pFeatureLevels: *const u32,
    FeatureLevels: u32,
    SDKVersion: u32,
    ppDevice: *mut *mut c_void,
    pFeatureLevel: *mut u32,
    ppImmediateContext: *mut *mut std::ffi::c_void
) -> i32;

#[no_mangle]
pub unsafe extern "system" fn f_create_device(
    orig_func: usize,
    pAdapter: *mut c_void,
    DriverType: u32,
    Software: *mut c_void,
    Flags: u32,
    pFeatureLevels: *const u32,
    FeatureLevels: u32,
    SDKVersion: u32,
    ppDevice: *mut *mut c_void,
    pFeatureLevel: *mut u32,
    ppImmediateContext: *mut *mut c_void
) -> i32{
    log!("D3D11CreateDevice called");
    let orig_func: D3D11CreateDevice_fn = unsafe { std::mem::transmute(orig_func) };
    let result = unsafe {
        orig_func(
            pAdapter,
            DriverType,
            Software,
            Flags,
            pFeatureLevels,
            FeatureLevels,
            SDKVersion,
            ppDevice,
            pFeatureLevel,
            ppImmediateContext
        )
    };
    log!("D3D11CreateDevice Result: {}", result);
    log!("Creating device hooks");
//    install_device_hooks(*ppDevice);
    log!("VTable Hooks installed successfully.");
    result
}