use std::ffi::c_void;
use windows::Win32::Graphics::Direct3D11::*;
use windows_result::HRESULT;

pub type CreateBuffer = unsafe extern "system" fn(
    *mut c_void,
    *const D3D11_BUFFER_DESC,
    *const D3D11_SUBRESOURCE_DATA,
    *mut *mut ID3D11Buffer,
) -> HRESULT;

pub type CreateTexture2D = unsafe extern "system" fn(
    *mut c_void,
    *const D3D11_TEXTURE2D_DESC,
    *const D3D11_SUBRESOURCE_DATA,
    *mut *mut ID3D11Texture2D,
) -> HRESULT;

pub type CreateShaderResourceView = unsafe extern "system" fn(
    *mut c_void,
    *mut ID3D11Resource,
    *const D3D11_SHADER_RESOURCE_VIEW_DESC,
    *mut *mut ID3D11ShaderResourceView,
) -> HRESULT;

pub type CreateUnorderedAccessView = unsafe extern "system" fn(
    *mut c_void,
    *mut ID3D11Resource,
    *const D3D11_UNORDERED_ACCESS_VIEW_DESC,
    *mut *mut ID3D11UnorderedAccessView,
) -> HRESULT;

pub type CreateRenderTargetView = unsafe extern "system" fn(
    *mut c_void,
    *mut ID3D11Resource,
    *const D3D11_RENDER_TARGET_VIEW_DESC,
    *mut *mut ID3D11RenderTargetView,
) -> HRESULT;

pub type CreateDepthStencilView = unsafe extern "system" fn(
    *mut c_void,
    *mut ID3D11Resource,
    *const D3D11_DEPTH_STENCIL_VIEW_DESC,
    *mut *mut ID3D11DepthStencilView,
) -> HRESULT;

pub type CreateVertexShader = unsafe extern "system" fn(
    *mut c_void,
    *const c_void,
    usize,
    *mut ID3D11ClassLinkage,
    *mut *mut ID3D11VertexShader,
) -> HRESULT;

pub type CreatePixelShader = unsafe extern "system" fn(
    *mut c_void,
    *const c_void,
    usize,
    *mut ID3D11ClassLinkage,
    *mut *mut ID3D11PixelShader,
) -> HRESULT;

pub type CreateComputeShader = unsafe extern "system" fn(
    *mut c_void,
    *const c_void,
    usize,
    *mut ID3D11ClassLinkage,
    *mut *mut ID3D11ComputeShader,
) -> HRESULT;

pub type CreateDeferredContext = unsafe extern "system" fn(
    *mut c_void,
    u32,
    *mut *mut ID3D11DeviceContext,
) -> HRESULT;

pub type GetImmediateContext = unsafe extern "system" fn(
    *mut c_void,
    *mut *mut ID3D11DeviceContext,
);