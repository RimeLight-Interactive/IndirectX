use arc_swap::ArcSwap;
use rapidhash::RapidHashMap;
use windows::Win32::Graphics::Direct3D11::D3D11_MAP_WRITE_DISCARD;
use windows::core::Interface;
use windows_result::HRESULT;
use std::sync::Arc;
use std::sync::LazyLock;
use std::ffi::c_void;
use windows::Win32::Graphics::Direct3D11::{ID3D11Buffer, ID3D11Resource, D3D11_MAPPED_SUBRESOURCE as MSR, D3D11_MAP_WRITE_NO_OVERWRITE};
use crate::config::{CbvOverride, ShaderHex};
use crate::CONFIG;
use crate::hooks::context::{
    cs_get_constant_buffers::hooked_func as get_cs_cbv,
    vs_get_constant_buffers::hooked_func as get_vs_cbv,
    ps_get_constant_buffers::hooked_func as get_ps_cbv,
    map::hooked_func as map,
    unmap::hooked_func as unmap
};
use crate::log;


// ---------------------------------------------------------------------------
// Runtime Fast Types (Pre-flattened for Hot Path)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct FastOverride {
    pub offset: usize,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Default)]
pub struct FastCBPatch {
    pub active_slots_mask: u32,
    pub slot_patches: RapidHashMap<u32, Vec<FastOverride>>,
}

impl FastCBPatch {
    #[inline(always)]
    pub fn has_slot(&self, slot: u32) -> bool {
        if slot >= 32 {
            return false;
        }
        (self.active_slots_mask & (1u32 << slot)) != 0
    }
}

// ---------------------------------------------------------------------------
// CbvPatchManager Definition & Singleton
// ---------------------------------------------------------------------------

pub struct CbvPatchManager {
    patches: ArcSwap<RapidHashMap<u64, FastCBPatch>>,
}

impl CbvPatchManager {
    fn new() -> Self {
        Self {
            patches: ArcSwap::from_pointee(RapidHashMap::default()),
        }
    }

    #[inline(always)]
    pub fn get_patches(&self, shader_hash: u64) -> Option<FastCBPatch> {
        let current = self.patches.load();
        let cb_patch = current.get(&shader_hash);
        if cb_patch.is_none() {
            return None;
        }
        cb_patch.cloned()
    }
        


    pub fn reload_and_parse(&self) {
        let config_guard = match CONFIG.get() {
            Some(c) => c.load(),
            None => return,
        };

        let mut new_map: RapidHashMap<u64, FastCBPatch> = RapidHashMap::default();

        // Use RapidHashMap here to match ShaderConfig's cbv_patch type!
        let mut process_stage = |stage_patches: Option<&RapidHashMap<ShaderHex, Vec<CbvOverride>>>| {
            if let Some(patches_map) = stage_patches {
                for (shader_hex, overrides) in patches_map {
                    let hash = shader_hex.0;
                    let shader_entry = new_map.entry(hash).or_default();

                    for rule in overrides {
                        let slot_u32 = rule.slot as u32;

                        if slot_u32 < 32 {
                            shader_entry.active_slots_mask |= 1u32 << slot_u32;
                        }

                        let slot_overrides = shader_entry.slot_patches.entry(slot_u32).or_default();
                        slot_overrides.push(FastOverride {
                            offset: rule.offset,
                            bytes: rule.value.to_bytes(),
                        });
                    }
                }
            }
        };

        process_stage(config_guard.pixel_shaders.cbv_patch.as_ref());
        process_stage(config_guard.vertex_shaders.cbv_patch.as_ref());
        process_stage(config_guard.compute_shaders.cbv_patch.as_ref());

        self.patches.swap(Arc::new(new_map));
        crate::log!("[+] [CBVPatchManager] Successfully reloaded all stage CBV patches.");
    }
}

// Global Singleton
pub static CBV_PATCH_MANAGER: LazyLock<CbvPatchManager> = LazyLock::new(CbvPatchManager::new);

pub fn reload_and_parse() {
    CBV_PATCH_MANAGER.reload_and_parse();
}

pub fn get_patches(shader_hash: u64) -> Option<FastCBPatch> {
    CBV_PATCH_MANAGER.get_patches(shader_hash)
}

pub unsafe fn apply_patches_cs(context: *mut c_void, patches:&FastCBPatch){
    for (&slot, patch) in patches.slot_patches.iter() {
        let mut p_cbv = std::ptr::null_mut();
        get_cs_cbv(context, slot, 1, &mut p_cbv);
        if p_cbv.is_null() {
            log!("Failed to fetch cbv to patch! skipping...");
            continue;
        }
        let buffer = ID3D11Buffer::from_raw(p_cbv as *mut c_void);
        let mut buffer_resource: ID3D11Resource = buffer.cast().unwrap_or_else(|_|{
            log!("Casting failed");
            std::process::exit(1);
            }
        );
        log!("Calling map!");
        let mut mapped_resource: MSR = MSR::default();
        let result = map(context, 
            p_cbv as *mut ID3D11Resource,
            0, 
            D3D11_MAP_WRITE_NO_OVERWRITE,
            0,  
            &mut mapped_resource
        );

        if result != HRESULT(0){
            log!("Failed to map cbv to writable. skipping...");
            continue;
        }
        let data_ptr = mapped_resource.pData;
        log!("performing surgery");
        for rule in patch.iter() {
            std::ptr::copy_nonoverlapping(
                rule.bytes.as_ptr(),
                (data_ptr as *mut u8).add(rule.offset),
                rule.bytes.len(),
            );
        }
        log!("unmapping");
        unmap(context, p_cbv as *mut ID3D11Resource, 0);
        log!("done");
    }
}

pub unsafe fn apply_patches_vs(context: *mut c_void, patches:&FastCBPatch){
    for (&slot, patch) in patches.slot_patches.iter() {
        let mut p_cbv = std::ptr::null_mut();
        get_vs_cbv(context, slot, 1, &mut p_cbv);
        if !p_cbv.is_null() {
            let mut buffer = ID3D11Resource::from_raw(p_cbv as *mut c_void);
            let mut mapped_resource: MSR = std::mem::zeroed();
            let result = map(context, &mut buffer,
                0, 
                D3D11_MAP_WRITE_DISCARD,
                0,  
                &mut mapped_resource
            );

            if result != HRESULT(0){
                continue;
            }
            let data_ptr = mapped_resource.pData;
            for rule in patch.iter() {
                std::ptr::copy_nonoverlapping(
                    rule.bytes.as_ptr(),
                    (data_ptr as *mut u8).add(rule.offset),
                    rule.bytes.len(),
                );
            }
            unmap(context, &mut buffer, 0);
        }
    }
}

pub unsafe fn apply_patches_ps(context: *mut c_void, patches:&FastCBPatch){
    for (&slot, patch) in patches.slot_patches.iter() {
        let mut p_cbv = std::ptr::null_mut();
        get_ps_cbv(context, slot, 1, &mut p_cbv);
        if !p_cbv.is_null() {
            let mut buffer = ID3D11Resource::from_raw(p_cbv as *mut c_void);
            let mut mapped_resource: MSR = std::mem::zeroed();
            let result = map(context, &mut buffer,
                0, 
                D3D11_MAP_WRITE_NO_OVERWRITE,
                0,  
                &mut mapped_resource
            );

            if result != HRESULT(0){
                continue;
            }
            let data_ptr = mapped_resource.pData;
            for rule in patch.iter() {
                std::ptr::copy_nonoverlapping(
                    rule.bytes.as_ptr(),
                    (data_ptr as *mut u8).add(rule.offset),
                    rule.bytes.len(),
                );
            }
            unmap(context, &mut buffer, 0);
        }
    }
}