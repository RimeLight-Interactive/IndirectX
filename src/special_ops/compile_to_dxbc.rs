use std::{
    path::Path,
    fs
};
use std::ffi::c_void;
use windows::Win32::Graphics::Direct3D::Fxc::{
    D3DCompile, D3DCOMPILE_ENABLE_STRICTNESS, D3DCOMPILE_OPTIMIZATION_LEVEL3,
};
use windows::Win32::Graphics::Direct3D::ID3DBlob;
use windows::Win32::Graphics::Direct3D11::{ID3D11ComputeShader, ID3D11PixelShader};
use windows::core::PCSTR;
use d3dasm;
use dxbc::{ChunkData, Program, build_dxbc, chunks::WritableChunk};
use windows::Win32::Graphics::Direct3D11::D3D11_AES_CTR_IV;
use crate::log;
use super::shader_patcher::patch_dxbc_checksum;

pub fn shdr_to_dxbc(original: Vec<u8>, shdr: &Path) -> Option<Vec<u8>> {
    let shdr_dxbc = fs::read(shdr);
    if shdr_dxbc.is_err() {
        return None;
    }
    let shdr_dxbc = shdr_dxbc.unwrap();
    let orig_shader_vec = d3dasm::parse(&original);
    let orig_shader = &orig_shader_vec[0];
    let modded_shader_vec = d3dasm::parse(&shdr_dxbc);
    let modded_shader = &modded_shader_vec[0];
    let mut modded_shader_program: Option<WritableChunk> = None;
    for chunk in modded_shader.chunks() {
        if let ChunkData::Shader(program) = chunk {
            modded_shader_program = chunk.to_writable();
        }
    }
    if modded_shader_program.is_none() {
        return None;
    }
    let mut writable_chunks: Vec<WritableChunk> = vec![];
    for chunk in orig_shader.chunks() {
        if let ChunkData::Shader(program) = chunk {
            writable_chunks.push(modded_shader_program.clone().unwrap());
            continue;
        }
        writable_chunks.push(chunk.to_writable().unwrap());
    }
    let mut modded_dxbc = build_dxbc(&writable_chunks);
    patch_dxbc_checksum(&mut modded_dxbc);
    Some(modded_dxbc)
}

pub fn hlsl_to_dxbc(source_path: &Path, target_profile: &[u8]) -> Option<Vec<u8>> {
    let source_code = match std::fs::read_to_string(source_path) {
        Ok(code) => code,
        Err(e) => {
            log!("[!] [ShaderCompiler] Failed to read {:?}: {}", source_path, e);
            return None;
        }
    };

    let source_name = source_path.to_str().unwrap_or("shader.hlsl");
    let source_name_cstr = std::ffi::CString::new(source_name).unwrap_or_default();

    let mut code_blob: Option<ID3DBlob> = None;
    let mut error_blob: Option<ID3DBlob> = None;

    let flags = D3DCOMPILE_ENABLE_STRICTNESS | D3DCOMPILE_OPTIMIZATION_LEVEL3;

    let result = unsafe {
        D3DCompile(
            source_code.as_ptr() as *const c_void,
            source_code.len(),
            PCSTR::from_raw(source_name_cstr.as_ptr() as *const u8),
            None,
            None,
            PCSTR::from_raw(b"main\0".as_ptr()),
            PCSTR::from_raw(target_profile.as_ptr()),
            flags,
            0,
            &mut code_blob as *mut Option<ID3DBlob>,
            Some(&mut error_blob as *mut Option<ID3DBlob>),
        )
    };

    if let Err(err) = result {
        if let Some(err_blob) = error_blob {
            let msg = unsafe {
                let ptr = err_blob.GetBufferPointer() as *const u8;
                let size = err_blob.GetBufferSize();
                String::from_utf8_lossy(std::slice::from_raw_parts(ptr, size)).into_owned()
            };
            log!("[!] [ShaderCompiler] Compile error in {:?}:\n{}", source_path, msg);
        } else {
            log!("[!] [ShaderCompiler] Unknown compile failure ({:?}) for {:?}", err, source_path);
        }
        return None;
    }

    let code_blob = code_blob?;
    let bytecode = unsafe {
        let ptr = code_blob.GetBufferPointer() as *const u8;
        let size = code_blob.GetBufferSize();
        std::slice::from_raw_parts(ptr, size).to_vec()
    };

    Some(bytecode)
}