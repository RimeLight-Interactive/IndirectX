use crate::fn_typedefs::device::CreateComputeShader;
use std::result;
use std::sync::OnceLock;
use std::ffi::c_void;
use windows::Win32::Graphics::Direct3D11::*;
use windows_result::HRESULT;
use crate::special_ops::shader_manager::register_new_cs;
use crate::helpers::hash;

static ORIG_FUNC: OnceLock<CreateComputeShader> = OnceLock::new();

pub fn set_orig_func(func: usize) {
    let _ = ORIG_FUNC.set(unsafe { std::mem::transmute(func) });
}

pub fn hooked_func(
    this: *mut c_void,
    p_shader_bytecode: *const c_void,
    len: usize,
    classlinkage: *mut c_void,
    computeshader: *mut *mut ID3D11ComputeShader,
) -> HRESULT {
    let result;
    unsafe {
        let func = ORIG_FUNC.get().unwrap();
        result = func(this, p_shader_bytecode, len, classlinkage, computeshader);

        // Check 1: Function succeeded and pointer array exists
        if result >= HRESULT(0) && !computeshader.is_null() && !p_shader_bytecode.is_null() {
            let created_shader = *computeshader;

            if !created_shader.is_null() {
                register_new_cs(
                    p_shader_bytecode as *const u8,
                    len,
                    created_shader as usize
                );
            }
        }
    }
    result
}


pub fn create_sub_cs(
    this: *mut c_void,
    shader_bytecode: &[u8],
) -> *mut ID3D11ComputeShader {
    // 1. Safety Check: Don't call into DX11 with an empty slice
    if shader_bytecode.is_empty() || this.is_null() {
        return std::ptr::null_mut();
    }

    let orig_fn = match ORIG_FUNC.get() {
        Some(f) => f,
        None => return std::ptr::null_mut(),
    };

    // 2. Allocate out-pointer on our stack to avoid dangling pointer issues
    let mut created_shader: *mut ID3D11ComputeShader = std::ptr::null_mut();

    unsafe {
        let result = orig_fn(
            this,
            shader_bytecode.as_ptr() as *const c_void,
            shader_bytecode.len(),
            std::ptr::null_mut(), // pClassLinkage = NULL (completely fine!)
            &mut created_shader as *mut *mut ID3D11ComputeShader,
        );

        // 3. Proper COM success check (SUCCEEDED macro equivalent)
        if result.is_ok() && !created_shader.is_null() {
            created_shader
        } else {
            // Log if useful: crate::log!("[!] CreateComputeShader failed: {:?}", result);
            std::ptr::null_mut()
        }
    }
}


