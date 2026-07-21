#[allow(unused)]
use std::ffi::c_void;
use windows::Win32::Graphics::{
    Direct3D11::*,
    Dxgi::Common::DXGI_FORMAT};
use windows_result::HRESULT;
use windows::core::BOOL;

pub type PSSetShader = unsafe extern "system" fn(
    *mut c_void,
    *mut ID3D11PixelShader,
    *const *mut ID3D11ClassInstance,
    u32,
);

pub type VSSetShader = unsafe extern "system" fn(
    *mut c_void,
    *mut ID3D11VertexShader,
    *const *mut ID3D11ClassInstance,
    u32,
);

pub type DrawIndexed = unsafe extern "system" fn(
    *mut c_void,
    u32,
    u32,
    i32,
);

pub type Draw = unsafe extern "system" fn(
    *mut c_void,
    u32,
    u32,
);

pub type DrawIndexedInstanced = unsafe extern "system" fn(
    *mut c_void,
    u32,
    u32,
    u32,
    i32,
    u32,
);

pub type OMSetRenderTargets = unsafe extern "system" fn(
    *mut c_void,
    u32,
    *const *mut ID3D11RenderTargetView,
    *mut ID3D11DepthStencilView,
);

pub type OMSetRenderTargetsAndUnorderedAccessViews = unsafe extern "system" fn(
    *mut c_void,
    u32,
    *const *mut ID3D11RenderTargetView,
    *mut ID3D11DepthStencilView,
    u32,
    u32,
    *const *mut ID3D11UnorderedAccessView,
    *const u32,
);

pub type Dispatch = unsafe extern "system" fn(
    *mut c_void,
    u32,
    u32,
    u32,
);

pub type DispatchIndirect = unsafe extern "system" fn(
    *mut c_void,
    *mut ID3D11Buffer,
    u32,
);

pub type RSSetViewports = unsafe extern "system" fn(
    *mut c_void,
    u32,
    *const D3D11_VIEWPORT,
);

pub type CopySubresourceRegion = unsafe extern "system" fn(
    *mut c_void,
    *mut ID3D11Resource,
    u32,
    u32,
    u32,
    u32,
    *mut ID3D11Resource,
    u32,
    *const D3D11_BOX,
);

pub type CopyResource = unsafe extern "system" fn(
    *mut c_void,
    *mut ID3D11Resource,
    *mut ID3D11Resource,
);

pub type UpdateSubresource = unsafe extern "system" fn(
    *mut c_void,
    *mut ID3D11Resource,
    u32,
    *const D3D11_BOX,
    *const c_void,
    u32,
    u32,
);

pub type GenerateMips = unsafe extern "system" fn(
    *mut c_void,
    *mut ID3D11ShaderResourceView,
);

pub type ResolveSubresource = unsafe extern "system" fn(
    *mut c_void,
    *mut ID3D11Resource,
    u32,
    *mut ID3D11Resource,
    u32,
    DXGI_FORMAT,
);

pub type ExecuteCommandList = unsafe extern "system" fn(
    *mut c_void,
    *mut ID3D11CommandList,
    BOOL,
);

pub type CSSetShaderResources = unsafe extern "system" fn(
    *mut c_void,
    u32,
    u32,
    *const *mut ID3D11ShaderResourceView,
);

pub type CSSetUnorderedAccessViews = unsafe extern "system" fn(
    *mut c_void,
    u32,
    u32,
    *const *mut ID3D11UnorderedAccessView,
    *const u32,
);

pub type CSSetShader = unsafe extern "system" fn(
    *mut c_void,
    *mut ID3D11ComputeShader,
    *const *mut ID3D11ClassInstance,
    u32,
);

pub type CSSetConstantBuffers = unsafe extern "system" fn(
    *mut c_void,
    u32,
    u32,
    *const *mut ID3D11Buffer,
);

pub type FinishCommandList = unsafe extern "system" fn(
    *mut c_void,
    BOOL,
    *mut *mut ID3D11CommandList,
) -> HRESULT;

pub type Map = unsafe extern "system" fn(
    *mut c_void,
    *mut ID3D11Resource,
    u32,
    D3D11_MAP,
    u32,
    *mut D3D11_MAPPED_SUBRESOURCE,
) -> HRESULT;

pub type Unmap = unsafe extern "system" fn(
    *mut c_void,
    *mut ID3D11Resource,
    u32,
);

pub type PSSetShaderResources = unsafe extern "system" fn(
    *mut c_void,
    u32,
    u32,
    *const *mut ID3D11ShaderResourceView,
);

pub type PSSetConstantBuffers = unsafe extern "system" fn(
    *mut c_void,
    u32,
    u32,
    *const *mut ID3D11Buffer,
);

pub type VSSetConstantBuffers = unsafe extern "system" fn(
    *mut c_void,
    u32,
    u32,
    *const *mut ID3D11Buffer,
);

pub type IASetVertexBuffers = unsafe extern "system" fn(
    *mut c_void,
    u32,
    u32,
    *const *mut ID3D11Buffer,
    *const u32,
    *const u32,
);

pub type IASetIndexBuffer = unsafe extern "system" fn(
    *mut c_void,
    *mut ID3D11Buffer,
    DXGI_FORMAT,
    u32,
);

pub type ClearRenderTargetView = unsafe extern "system" fn(
    *mut c_void,
    *mut ID3D11RenderTargetView,
    *const f32,
);

pub type ClearDepthStencilView = unsafe extern "system" fn(
    *mut c_void,
    *mut ID3D11DepthStencilView,
    D3D11_CLEAR_FLAG,
    f32,
    u8,
);
