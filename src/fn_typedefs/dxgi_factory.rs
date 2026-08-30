#[allow(unused)]
use std::ffi::c_void;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Dxgi::*;
use windows_result::HRESULT;

pub type CreateSwapChain = unsafe extern "system" fn(
    *mut c_void,
    *mut c_void,
    *const DXGI_SWAP_CHAIN_DESC,
    *mut *mut c_void,
) -> HRESULT;

pub type CreateSwapChainForHwnd = unsafe extern "system" fn(
    *mut c_void,
    *mut c_void,
    HWND,
    *const DXGI_SWAP_CHAIN_DESC1,
    *const DXGI_SWAP_CHAIN_FULLSCREEN_DESC,
    *mut c_void,
    *mut *mut c_void,
) -> HRESULT;

pub type CreateSwapChainForCoreWindow = unsafe extern "system" fn(
    *mut c_void,
    *mut c_void,
    *mut c_void,
    *const DXGI_SWAP_CHAIN_DESC1,
    *mut c_void,
    *mut *mut c_void,
) -> HRESULT;

pub type CreateSwapChainForComposition = unsafe extern "system" fn(
    *mut c_void,
    *mut c_void,
    *const DXGI_SWAP_CHAIN_DESC1,
    *mut c_void,
    *mut *mut c_void,
) -> HRESULT;