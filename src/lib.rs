
mod config;
mod logger;
mod exports;
mod proxy;
mod hooks;
mod fn_typedefs;
mod talker;
mod helpers;
mod special_ops;

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::sync::OnceLock;

use arc_swap::ArcSwap;

use windows::Win32::Foundation::HMODULE;
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;
use windows::core::*;

use crate::config::Config;

static mut DLL: Option<HMODULE> = None;
static CONFIG: OnceLock<ArcSwap<Config>> = OnceLock::new();

pub unsafe extern "system" fn f_main(
    _dll_module: usize,
    call_reason: u32,
    _reserved: usize,
) -> bool {
    if call_reason != DLL_PROCESS_ATTACH {
        return true;
    }
    let _ = CONFIG.set(ArcSwap::from_pointee(Config::load()));
    let config = CONFIG.get().unwrap().load();
    let _ = logger::init("IndirectX.log", config.logging, config.log_async);
    log!("IndirectX started!");
    let dll_name = config.next_dll.as_ref().unwrap();
    let wide: Vec<u16> = OsStr::new(&dll_name)
        .encode_wide()
        .chain(Some(0))
        .collect();
    log!("Loading next DLL: {}", dll_name);
    DLL = Some(
        LoadLibraryW(PCWSTR(wide.as_ptr()))
            .expect("Unable to load Next DLL"),
    );
    log!("Next DLL loaded successfully.");
    log!("Setting up exports...");
    let symbols = [
    "D3D11CoreCreateDevice",
    "D3D11CreateDeviceAndSwapChain",
    "D3D11On12CreateDevice",
    "D3D11CreateDevice",
    ];
    
    for symbol in symbols.iter() {
        let symbol_cstr = std::ffi::CString::new(*symbol).unwrap();
        let symbol_ptr = GetProcAddress(DLL.unwrap(), PCSTR(symbol_cstr.as_ptr() as *const u8))
            .unwrap_or_else(|| panic!("Unable to get address of {}", symbol));
        let symbol_addr = symbol_ptr as usize;

        match *symbol {
            "D3D11CoreCreateDevice" => exports::set_D3D11CoreCreateDevice_orig(symbol_addr),
            "D3D11CreateDeviceAndSwapChain" =>
                exports::set_D3D11CreateDeviceAndSwapChain_orig(symbol_addr),
            "D3D11On12CreateDevice" => exports::set_D3D11On12CreateDevice_orig(symbol_addr),
            "D3D11CreateDevice" => exports::set_D3D11CreateDevice_orig(symbol_addr),
            _ => log!("A Wild Symbol appeared!: {}", symbol),
        }
    }
    log!("Exports linked!");
    log!("Attempting to start server");
    std::thread::spawn(talker::server_summoner::server_summoner);
    true
}



