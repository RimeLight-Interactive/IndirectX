mod exports;

use std::{ffi::OsStr, os::windows::ffi::OsStrExt};
use windows::{Win32::System::LibraryLoader::*, core::{PCSTR, PCWSTR}};

pub const DLL_PROCESS_ATTACH: u32 = 1;

#[unsafe(no_mangle)]
pub unsafe extern "system" fn DllMain(
        _dll_module: usize,
    call_reason: u32,
    _reserved: usize,
) -> i32 {
    if call_reason != DLL_PROCESS_ATTACH {
        return 1;
    }
    let dll_name_wide: Vec<u16> = OsStr::new("IndirectX.dll")
    .encode_wide()
    .chain(Some((0)))
    .collect();

    let indirectx_dll = Some(
        LoadLibraryW(PCWSTR(dll_name_wide.as_ptr())).unwrap()
    ).unwrap();
    let symbols = [
    "D3D11CoreCreateDevice",
    "D3D11CreateDeviceAndSwapChain",
    "D3D11On12CreateDevice",
    "D3D11CreateDevice",
    ];
    
    for symbol in symbols.iter() {
        let symbol_cstr = std::ffi::CString::new(*symbol).unwrap();
        let symbol_ptr = GetProcAddress(indirectx_dll, PCSTR(symbol_cstr.as_ptr() as *const u8))
            .unwrap_or_else(|| panic!("Unable to get address of {}", symbol));
        let symbol_addr = symbol_ptr as usize;

        match *symbol {
            "D3D11CoreCreateDevice" => exports::set_D3D11CoreCreateDevice_orig(symbol_addr),
            "D3D11CreateDeviceAndSwapChain" =>
                exports::set_D3D11CreateDeviceAndSwapChain_orig(symbol_addr),
            "D3D11On12CreateDevice" => exports::set_D3D11On12CreateDevice_orig(symbol_addr),
            "D3D11CreateDevice" => exports::set_D3D11CreateDevice_orig(symbol_addr),
            _ => continue
        }
    };
    1
}

