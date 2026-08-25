use crate::fn_typedefs::context::PSSetShader;
use crate::special_ops::shader_manager::{get_ps_replacement, set_active_ps};
use std::ffi::c_void;
use std::sync::OnceLock;
use windows::Win32::Graphics::Direct3D11::{ID3D11ClassInstance, ID3D11PixelShader};

static ORIG_FUNC: OnceLock<PSSetShader> = OnceLock::new();

pub fn set_orig_func(func: usize) {
    let _ = ORIG_FUNC.set(unsafe { std::mem::transmute(func) });
}

pub fn hooked_func(
    this: *mut c_void,
    pixelshader: *mut ID3D11PixelShader,
    classinstance: *const *mut ID3D11ClassInstance,
    a: u32,
) {
    unsafe {
        let func= ORIG_FUNC.get().unwrap();
        let ctx_key = this as usize;
        let original_shader = pixelshader;

        // 1. First, track the incoming original shader for this context
        set_active_ps(ctx_key, original_shader as usize);

        // 2. Now query for a replacement (resolves via the newly tracked original shader)
        let mut final_shader = original_shader;
        if let Some(replacement_shader) = get_ps_replacement(ctx_key) {
            final_shader = replacement_shader as *mut ID3D11PixelShader;
        }
        
        func(this, final_shader, classinstance, a);
    }
}