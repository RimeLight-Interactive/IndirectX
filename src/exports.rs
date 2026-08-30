use core::arch::naked_asm;
use windows::core::GUID;
use paste::paste;
use std::ffi::c_void;

use crate::log;


macro_rules! naked_trampoline {
    ($name:ident) => {
        paste! {
            #[allow(non_upper_case_globals)]
            static mut [<$name _ORIG_PTR>]: usize = 0;

            #[allow(dead_code)]
            #[inline(always)]
            pub unsafe fn [<set_ $name _orig>](ptr: usize) {
                [<$name _ORIG_PTR>] = ptr;
            }

            #[no_mangle]
            #[unsafe(naked)]
            pub unsafe extern "system" fn $name() -> ! {
                naked_asm!(
                    "mov rax, qword ptr [rip + {target}]",
                    "jmp rax",
                    target = sym [<$name _ORIG_PTR>],
                );
            }
        }
    };
}

naked_trampoline!(D3D11CoreCreateDevice);
naked_trampoline!(D3D11On12CreateDevice);
naked_trampoline!(DXGIDeclareAdapterRemovalSupport);
naked_trampoline!(DXGIGetDebugInterface1);

// Main exports
#[no_mangle]
pub unsafe extern "system" fn DllMain(
    _dll_module: usize,
    call_reason: u32,
    _reserved: usize,
) -> i32 {
    crate::f_main(_dll_module, call_reason, _reserved) as i32
}

static mut D3_D11_CREATE_DEVICE_ORIG_PTR: usize = 0;
pub unsafe fn set_D3D11CreateDevice_orig(ptr: usize) {
    unsafe {
        D3_D11_CREATE_DEVICE_ORIG_PTR = ptr;
    }
}

#[no_mangle]
pub unsafe extern "system" fn D3D11CreateDevice(
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

) -> i32 {
    log!("Create Device Called");
    crate::proxy::f_create_device(
        D3_D11_CREATE_DEVICE_ORIG_PTR,
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
}

static mut D3_D11_CREATE_DEVICE_AND_SWAPCHAIN_ORIG_PTR: usize = 0;
pub unsafe fn set_D3D11CreateDeviceAndSwapChain_orig(ptr: usize) {
    unsafe {
        D3_D11_CREATE_DEVICE_AND_SWAPCHAIN_ORIG_PTR = ptr;
    }
}

#[no_mangle]
pub unsafe extern "system" fn D3D11CreateDeviceAndSwapChain(
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

) -> i32 {
    crate::proxy::f_create_device_and_swapchain(
        D3_D11_CREATE_DEVICE_AND_SWAPCHAIN_ORIG_PTR,
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
}

static mut CREATE_DXGI_FACTORY_ORIG_PTR: usize = 0;
pub unsafe fn set_CreateDXGIFactory_orig(ptr: usize) {
    unsafe {
        CREATE_DXGI_FACTORY_ORIG_PTR = ptr;
    }
}
static mut CREATE_DXGI_FACTORY1_ORIG_PTR: usize = 0;
pub unsafe fn set_CreateDXGIFactory1_orig(ptr: usize) {
    unsafe {
        CREATE_DXGI_FACTORY1_ORIG_PTR = ptr;
    }
}
static mut CREATE_DXGI_FACTORY2_ORIG_PTR: usize = 0;
pub unsafe fn set_CreateDXGIFactory2_orig(ptr: usize) {
    unsafe {
        CREATE_DXGI_FACTORY2_ORIG_PTR = ptr;
    }
}
#[no_mangle]
pub unsafe extern "system" fn CreateDXGIFactory(
    refiid: *const GUID,
    pp_factory: *mut *mut c_void
) -> i32 {
    crate::proxy::f_create_dxgi_factory(CREATE_DXGI_FACTORY_ORIG_PTR, refiid, pp_factory)
}

#[no_mangle]
pub unsafe extern "system" fn CreateDXGIFactory1(
    refiid: *const GUID,
    pp_factory: *mut *mut c_void
) -> i32 {
    crate::proxy::f_create_dxgi_factory1(CREATE_DXGI_FACTORY1_ORIG_PTR, refiid, pp_factory)
}

#[no_mangle]
pub unsafe extern "system" fn CreateDXGIFactory2(
    flags: u32,
    refiid: *const GUID,
    pp_factory: *mut *mut c_void
) -> i32 {
    crate::proxy::f_create_dxgi_factory2(CREATE_DXGI_FACTORY2_ORIG_PTR, flags, refiid, pp_factory)
}