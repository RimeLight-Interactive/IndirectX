use crate::fn_typedefs::context::CSSetShader;
use crate::special_ops::shader_manager::{get_cs_replacement, set_active_cs};
use std::ffi::c_void;
use std::sync::OnceLock;
use windows::Win32::Graphics::Direct3D11::{ID3D11ClassInstance, ID3D11ComputeShader};

static ORIG_FUNC: OnceLock<CSSetShader> = OnceLock::new();

pub fn set_orig_func(func: usize) {
    let _ = ORIG_FUNC.set(unsafe { std::mem::transmute(func) });
}

pub fn hooked_func(
    this: *mut c_void,
    computeshader: *mut ID3D11ComputeShader,
    classinstance: *const *mut ID3D11ClassInstance,
    a: u32,
) {
    unsafe {
        let func = ORIG_FUNC.get().unwrap();
        let ctx_key = this as usize;
        let original_shader = computeshader;

        set_active_cs(ctx_key, original_shader as usize);

        let mut final_shader = original_shader;
        if let Some(replacement_shader) = get_cs_replacement(ctx_key) {
            final_shader = replacement_shader as *mut ID3D11ComputeShader;
        }
        func(this, final_shader, classinstance, a);

    }
}