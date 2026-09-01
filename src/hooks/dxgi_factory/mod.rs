pub mod create_swapchain;
pub mod create_swapchain_for_hwnd;
pub mod create_swapchain_for_core_window;
pub mod create_swapchain_for_composition;
pub mod query_interface;

use std::ffi::c_void;

use crate::make_hook_map;

pub fn install_dxgi_factory_hooks(com: *mut c_void) {
    let hook_map = make_hook_map!(
        (0, query_interface),
        (10, create_swapchain),
    );

    super::install_hooks(com, &hook_map);
}

pub fn install_dxgi_factory2_hooks(com: *mut c_void) {
    let hook_map = make_hook_map!(
        (15, create_swapchain_for_hwnd),
        (16, create_swapchain_for_core_window),
        (24, create_swapchain_for_composition),
    );

    super::install_hooks(com, &hook_map);
}