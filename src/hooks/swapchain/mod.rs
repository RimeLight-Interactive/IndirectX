pub mod present;
pub mod resize_buffers;

use std::ffi::c_void;
use crate::make_hook_map;

pub fn install_swapchain_hooks(com: *mut c_void) {
    let hook_map = make_hook_map!(
        (8, present),
        (13, resize_buffers),
    );
    super::install_hooks(com, &hook_map);
}
