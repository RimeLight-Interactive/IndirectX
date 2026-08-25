use crate::fn_typedefs::device::CreatePixelShader;
use crate::special_ops::shader_patcher;
use std::sync::OnceLock;
use std::ffi::c_void;
use windows::Win32::Graphics::Direct3D11::*;
use windows_result::HRESULT;
use crate::special_ops::shader_manager::register_new_ps;

static ORIG_FUNC: OnceLock<CreatePixelShader> = OnceLock::new();

pub fn set_orig_func(func: usize) {
    let _ = ORIG_FUNC.set(unsafe { std::mem::transmute(func) });
}

pub fn hooked_func(
    this: *mut c_void,
    p_shader_bytecode: *const c_void,
    len: usize,
    classlinkage: *mut c_void,
    pixelshader: *mut *mut ID3D11PixelShader,
) -> HRESULT {
    let result;
    unsafe {
        let func = ORIG_FUNC.get().unwrap();

        result = func(this, p_shader_bytecode, len, classlinkage, pixelshader);

        // Check 1: Function succeeded and pointer array exists
        if result >= HRESULT(0) && !pixelshader.is_null() && !p_shader_bytecode.is_null() {
            let created_shader = *pixelshader;

            // Check 2: Ensure the underlying COM object pointer itself isn't null
            if !created_shader.is_null() {
                register_new_ps(
                     p_shader_bytecode as *const u8,
                    len,
                    created_shader as usize,
                );
            }
        }
    }
    result
}

pub fn create_sub_ps(
    this: *mut c_void,
    shader_bytecode: &[u8],
) -> *mut ID3D11PixelShader {
    // 1. Safety Check: Guard against null device or empty bytecode payload
    if shader_bytecode.is_empty() || this.is_null() {
        return std::ptr::null_mut();
    }

    let orig_fn = match ORIG_FUNC.get() {
        Some(f) => f,
        None => return std::ptr::null_mut(),
    };

    // 2. Stack allocation for the output COM pointer
    let mut created_shader: *mut ID3D11PixelShader = std::ptr::null_mut();

    unsafe {
        let result = orig_fn(
            this,
            shader_bytecode.as_ptr() as *const c_void,
            shader_bytecode.len(),
            std::ptr::null_mut(), // pClassLinkage = NULL (No class linkage needed)
            &mut created_shader as *mut *mut ID3D11PixelShader,
        );

        // 3. SUCCEEDED(hr) check
        if result.is_ok() && !created_shader.is_null() {
            created_shader
        } else {
            // Optional: crate::log!("[!] CreatePixelShader failed with HRESULT: {:?}", result);
            std::ptr::null_mut()
        }
    }
}