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
    "DXGIGetDebugInterface1",
    "DXGIDeclareAdapterRemovalSupport",
    "CreateDXGIFactory",
    "CreateDXGIFactory1",
    "CreateDXGIFactory2"
    ];
    
    for symbol in symbols.iter() {
        let symbol_cstr = std::ffi::CString::new(*symbol).unwrap();
        let symbol_ptr = GetProcAddress(indirectx_dll, PCSTR(symbol_cstr.as_ptr() as *const u8))
            .unwrap_or_else(|| {panic!()});
        let symbol_addr = symbol_ptr as usize;

        match *symbol {
            "CreateDXGIFactory" => exports::set_CreateDXGIFactory_orig(symbol_addr),
            "CreateDXGIFactory1" =>
                exports::set_CreateDXGIFactory1_orig(symbol_addr),
            "CreateDXGIFactory2" => exports::set_CreateDXGIFactory2_orig(symbol_addr),
            "DXGIGetDebugInterface1" => exports::set_DXGIGetDebugInterface1_orig(symbol_addr),
            "DXGIDeclareAdapterRemovalSupport" => exports::set_DXGIDeclareAdapterRemovalSupport_orig(symbol_addr),
            _ => continue
        }
    };
    1
}

