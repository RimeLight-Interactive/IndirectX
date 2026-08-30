// Proxy functions for D3D11 exports
use crate::log;
use crate::hooks::{
    device::install_device_hooks,
    context::install_context_hooks,
    swapchain::install_swapchain_hooks,
    dxgi_factory::{install_dxgi_factory_hooks, install_dxgi_factory2_hooks}
};
use std::ffi::c_void;
use crate::special_ops::{
    shader_manager,
    cbv_patch_manager
};

type D3D11CreateDeviceFn = unsafe extern "system" fn(
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
    log!("D3D11CreateDevice called. Original Pointer {}", orig_func);
    let orig_func: D3D11CreateDeviceFn = unsafe { std::mem::transmute(orig_func) };
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
    if result == 0 {
        if !ppDevice.is_null() && !(*ppDevice).is_null() {
            log!("Creating device hooks");
            install_device_hooks(*ppDevice);
        }
        if !ppImmediateContext.is_null() && !(*ppImmediateContext).is_null() {
            log!("Creating context hooks");
            install_context_hooks(*ppImmediateContext);
        }
        log!("VTable Hooks installed successfully.");
    }
    log!("Loading replacement shaders");
    shader_manager::reload_replacement_shaders();
    log!("Replacement shaders loaded!");
    log!("Loading CBV patches");
    cbv_patch_manager::reload_and_parse();
    log!("Successfully loaded CBV patches");
    result
}

type D3D11CreateDeviceAndSwapChainFn = unsafe extern "system" fn(
    pAdapter: *mut c_void,
    DriverType: u32,
    Software: *mut c_void,
    Flags: u32,
    pFeatureLevels: *const u32,
    FeatureLevels: u32,
    SDKVersion: u32,
    pSwapChainDesc: *mut c_void,
    ppSwapChain: *mut *mut c_void,
    ppDevice: *mut *mut c_void,
    pFeatureLevel: *mut u32,
    ppImmediateContext: *mut *mut std::ffi::c_void
) -> i32;

#[no_mangle]
pub unsafe extern "system" fn f_create_device_and_swapchain(
    orig_func: usize,
    pAdapter: *mut c_void,
    DriverType: u32,
    Software: *mut c_void,
    Flags: u32,
    pFeatureLevels: *const u32,
    FeatureLevels: u32,
    SDKVersion: u32,
    pSwapChainDesc: *mut c_void,
    ppSwapChain: *mut *mut c_void,
    ppDevice: *mut *mut c_void,
    pFeatureLevel: *mut u32,
    ppImmediateContext: *mut *mut c_void
) -> i32{
    let orig_func: D3D11CreateDeviceAndSwapChainFn = unsafe { std::mem::transmute(orig_func) };
    let result = unsafe {
        orig_func(
            pAdapter,
            DriverType,
            Software,
            Flags,
            pFeatureLevels,
            FeatureLevels,
            SDKVersion,
            pSwapChainDesc,
            ppSwapChain,
            ppDevice,
            pFeatureLevel,
            ppImmediateContext
        )
    };
    if result == 0 {
        if !ppDevice.is_null() && !(*ppDevice).is_null() {
            log!("Creating device hooks");
            install_device_hooks(*ppDevice);
        }
        if !ppImmediateContext.is_null() && !(*ppImmediateContext).is_null() {
            log!("Creating context hooks");
            install_context_hooks(*ppImmediateContext);
        }
        if !ppSwapChain.is_null() && !(*ppSwapChain).is_null() {
            log!("Creating swapchain hooks");
            install_swapchain_hooks(*ppSwapChain);
        }
        log!("VTable Hooks installed successfully.");
    }
    result
}

type CreateDXGIFactoryFn = unsafe extern "system" fn(
        *const windows::core::GUID,
        *mut *mut c_void,
) -> i32;

type CreateDXGIFactoryFn1 = unsafe extern "system" fn(
        *const windows::core::GUID,
        *mut *mut c_void,
) -> i32;

type CreateDXGIFactoryFn2 = unsafe extern "system" fn(
        u32,
        *const windows::core::GUID,
        *mut *mut c_void,
) -> i32;

#[no_mangle]
pub unsafe fn f_create_dxgi_factory(
    orig_func: usize,
    refiid: *const windows::core::GUID,
    pp_factory: *mut *mut c_void
) -> i32 {
    log!("Attempting to create dxgi_factory type 0");
    let func: CreateDXGIFactoryFn = std::mem::transmute(orig_func);
    let result = func(refiid, pp_factory);
    install_dxgi_factory_hooks(*pp_factory);
    log!("installed factory hooks!");
    result
}

#[no_mangle]
pub unsafe fn f_create_dxgi_factory1(
    orig_func: usize,
    refiid: *const windows::core::GUID,
    pp_factory: *mut *mut c_void
) -> i32 {
    log!("Attempting to create dxgi_factory type 1");
    log!("CreateDXGIFactory1");
    log!("  refiid = {:?}", *refiid);
    log!("  pp_factory = {:?}", pp_factory);
    let func: CreateDXGIFactoryFn1 = std::mem::transmute(orig_func);
    let result = func(refiid, pp_factory);
    log!("returned result {}", result);
    if !pp_factory.is_null() {
        install_dxgi_factory_hooks(*pp_factory);
        log!("installed factory hooks!");
    }
    result
}

#[no_mangle]
pub unsafe fn f_create_dxgi_factory2(
    orig_func: usize,
    flags: u32,
    refiid: *const windows::core::GUID,
    pp_factory: *mut *mut c_void
) -> i32 {
    log!("Attempting to create dxgi_factory type 2");
    let func: CreateDXGIFactoryFn2 = std::mem::transmute(orig_func);
    let result = func(flags, refiid, pp_factory);
    install_dxgi_factory_hooks(*pp_factory);
    log!("installed factory hooks! round 1");
    install_dxgi_factory2_hooks(*pp_factory);
    log!("installed factory hooks! round 2");
    result
}