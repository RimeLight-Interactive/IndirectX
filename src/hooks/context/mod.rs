pub mod ps_set_shader;
pub mod vs_set_shader;
pub mod draw_indexed;
pub mod draw;
pub mod draw_indexed_instanced;
pub mod om_set_render_targets;
pub mod om_set_render_targets_and_unordered_access_views;
pub mod dispatch;
pub mod dispatch_indirect;
pub mod rs_set_viewports;
pub mod copy_subresource_region;
pub mod copy_resource;
pub mod update_subresource;
pub mod generate_mips;
pub mod resolve_subresource;
pub mod execute_command_list;
pub mod cs_set_shader_resources;
pub mod cs_set_unordered_access_views;
pub mod cs_set_shader;
pub mod cs_set_constant_buffers;
pub mod finish_command_list;
pub mod map;
pub mod unmap;
pub mod ps_set_shader_resources;
pub mod ps_set_constant_buffers;
pub mod vs_set_constant_buffers;
pub mod ia_set_vertex_buffers;
pub mod ia_set_index_buffer;
pub mod clear_render_target_view;
pub mod clear_depth_stencil_view;

use std::ffi::c_void;
use crate::make_hook_map;

pub fn install_context_hooks(com: *mut c_void) {
    let hook_map = make_hook_map!(
        (9, ps_set_shader),
        (11, vs_set_shader),
        (12, draw_indexed),
        (13, draw),
        (20, draw_indexed_instanced),
        (33, om_set_render_targets),
        (34, om_set_render_targets_and_unordered_access_views),
        (41, dispatch),
        (42, dispatch_indirect),
        (44, rs_set_viewports),
        (46, copy_subresource_region),
        (47, copy_resource),
        (48, update_subresource),
        (54, generate_mips),
        (57, resolve_subresource),
        (58, execute_command_list),
        (67, cs_set_shader_resources),
        (68, cs_set_unordered_access_views),
        (69, cs_set_shader),
        (71, cs_set_constant_buffers),
        (114, finish_command_list),
        (14, map),
        (15, unmap),
        (8, ps_set_shader_resources),
        (16, ps_set_constant_buffers),
        (7, vs_set_constant_buffers),
        (18, ia_set_vertex_buffers),
        (19, ia_set_index_buffer),
        (50, clear_render_target_view),
        (53, clear_depth_stencil_view),
    );
    super::install_hooks(com, &hook_map);
}
