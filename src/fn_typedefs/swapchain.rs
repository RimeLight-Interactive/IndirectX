use std::ffi::c_void;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows_result::HRESULT;

pub type Present = unsafe extern "system" fn(
    *mut c_void,
    u32,
    u32,
) -> HRESULT;

pub type ResizeBuffers = unsafe extern "system" fn(
    *mut c_void,
    u32,
    u32,
    u32,
    DXGI_FORMAT,
    u32,
) -> HRESULT;
