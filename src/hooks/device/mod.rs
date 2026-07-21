pub mod create_buffer;
pub mod create_texture_2d;
pub mod create_shader_resource_view;
pub mod create_unordered_access_view;
pub mod create_render_target_view;
pub mod create_depth_stencil_view;
pub mod create_vertex_shader;
pub mod create_pixel_shader;
pub mod create_compute_shader;
pub mod create_deferred_context;
pub mod get_immediate_context;

use std::ffi::c_void;
use crate::make_hook_map;

pub fn install_device_hooks(com: *mut c_void) {
    let hook_map = make_hook_map!(
        (3, create_buffer),
        (5, create_texture_2d),
        (7, create_shader_resource_view),
        (8, create_unordered_access_view),
        (9, create_render_target_view),
        (10, create_depth_stencil_view),
        (12, create_vertex_shader),
        (15, create_pixel_shader),
        (18, create_compute_shader),
        (27, create_deferred_context),
        (40, get_immediate_context),
    );
    super::install_hooks(com, &hook_map);
}
