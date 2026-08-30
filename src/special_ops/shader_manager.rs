use arc_swap::ArcSwap;
use rapidhash::RapidHashMap;
use serde::{Deserialize, Serialize};
use std::ffi::c_void;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use windows::core::{Interface, PCSTR};
use windows::Win32::Graphics::Direct3D::Fxc::{
    D3DCompile, D3DCOMPILE_ENABLE_STRICTNESS, D3DCOMPILE_OPTIMIZATION_LEVEL3,
};
use windows::Win32::Graphics::Direct3D::ID3DBlob;
use windows::Win32::Graphics::Direct3D11::{ID3D11ComputeShader, ID3D11PixelShader};
use xxhash_rust::xxh3::xxh3_64;

use crate::CONFIG;
use crate::config::ShaderHex;
use crate::helpers::get_device;
use crate::hooks::device::{
    create_compute_shader::create_sub_cs, create_pixel_shader::create_sub_ps,
};
use crate::log;
use crate::special_ops::shader_patcher;
use super::compile_to_dxbc::{
    shdr_to_dxbc,
    hlsl_to_dxbc
};

// ---------------------------------------------------------------------------
// Paths
// ---------------------------------------------------------------------------

const DUMP_BASE: &str = "./IndirectX_shaders/Dumps";
const REPLACE_BASE: &str = "./IndirectX_shaders/Replacements";

// ---------------------------------------------------------------------------
// Structs & Types
// ---------------------------------------------------------------------------

/// Thread-safe internal stats storage using lock-free atomics.
pub struct InternalShaderStats {
    pub bind_count: AtomicU64,
    pub last_bound_unix_ms: AtomicU64,
}

impl Default for InternalShaderStats {
    fn default() -> Self {
        Self {
            bind_count: AtomicU64::new(0),
            last_bound_unix_ms: AtomicU64::new(0),
        }
    }
}

/// Serializable DTO for Axum JSON responses.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShaderStats {
    pub bind_count: u64,
    pub last_bound_unix_ms: u64,
}

// ---------------------------------------------------------------------------
// 1. Shader Registry (Cold Path + Metadata)
// ---------------------------------------------------------------------------

pub struct ShaderRegistry {
    /// Shader pointer address → hash
    created: RwLock<RapidHashMap<usize, u64>>,
    /// Hash → raw DXBC bytecode
    bytecode: RwLock<RapidHashMap<u64, Vec<u8>>>,
    /// Hash → atomic stats
    stats: RwLock<RapidHashMap<u64, InternalShaderStats>>,
}

impl ShaderRegistry {
    fn new() -> Self {
        Self {
            created: RwLock::new(RapidHashMap::default()),
            bytecode: RwLock::new(RapidHashMap::default()),
            stats: RwLock::new(RapidHashMap::default()),
        }
    }

    /// Registers a new shader. Returns the xxh3 hash on success so callers
    /// can act on it (e.g. dump queue) without recomputing.
    pub fn register(
        &self,
        bytecode_ptr: *const u8,
        len: usize,
        shader_ptr: usize,
    ) -> Option<u64> {
        if bytecode_ptr.is_null() || len == 0 || shader_ptr == 0 {
            return None;
        }

        let bytecode_slice = unsafe { std::slice::from_raw_parts(bytecode_ptr, len) };
        let hash = xxh3_64(bytecode_slice);

        // 1. Store bytecode (write-once)
        if let Ok(mut bc) = self.bytecode.write() {
            bc.entry(hash).or_insert_with(|| bytecode_slice.to_vec());
        }

        // 2. Map pointer → hash
        if let Ok(mut created) = self.created.write() {
            created.insert(shader_ptr, hash);
        }

        // 3. Initialise atomic stats
        if let Ok(mut stats) = self.stats.write() {
            stats.entry(hash).or_default();
        }

        Some(hash)
    }

    #[inline(always)]
    pub fn record_bind(&self, hash: u64) {
        if let Ok(stats) = self.stats.read() {
            if let Some(stat) = stats.get(&hash) {
                stat.bind_count.fetch_add(1, Ordering::Relaxed);
                let now_ms = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0);
                stat.last_bound_unix_ms.store(now_ms, Ordering::Relaxed);
            }
        }
    }

    #[inline(always)]
    pub fn get_hash(&self, shader_ptr: usize) -> Option<u64> {
        self.created.read().ok()?.get(&shader_ptr).copied()
    }

    pub fn get_bytecode(&self, hash: u64) -> Option<Vec<u8>> {
        self.bytecode.read().ok()?.get(&hash).cloned()
    }

    pub fn get_all_hashes(&self) -> Vec<u64> {
        let guard = match self.created.read() {
            Ok(g) => g,
            Err(_) => return Vec::new(),
        };
        let mut hashes: Vec<u64> = guard.values().copied().collect();
        hashes.sort_unstable();
        hashes.dedup();
        hashes
    }

    pub fn get_stats(&self) -> RapidHashMap<u64, ShaderStats> {
        let mut out = RapidHashMap::default();
        if let Ok(stats) = self.stats.read() {
            for (&hash, internal) in stats.iter() {
                out.insert(
                    hash,
                    ShaderStats {
                        bind_count: internal.bind_count.load(Ordering::Relaxed),
                        last_bound_unix_ms: internal.last_bound_unix_ms.load(Ordering::Relaxed),
                    },
                );
            }
        }
        out
    }
}

// ---------------------------------------------------------------------------
// 2. Active Shader Tracker (Pure Hot Path)
// ---------------------------------------------------------------------------

pub struct ShaderTracker {
    /// Context address → bound shader address
    active: ArcSwap<RapidHashMap<usize, usize>>,
}

impl ShaderTracker {
    fn new() -> Self {
        Self {
            active: ArcSwap::from_pointee(RapidHashMap::default()),
        }
    }

    #[inline(always)]
    pub fn set_active(&self, registry: &ShaderRegistry, ctx: usize, ptr: usize) {
        self.active.rcu(|current| {
            let mut next = (**current).clone();
            next.insert(ctx, ptr);
            next
        });

        if ptr != 0 {
            if let Some(hash) = registry.get_hash(ptr) {
                registry.record_bind(hash);
            }
        }
    }

    #[inline(always)]
    pub fn get_active(&self, ctx: usize) -> Option<usize> {
        match self.active.load().get(&ctx) {
            Some(&ptr) if ptr != 0 => Some(ptr),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Static Singletons
// ---------------------------------------------------------------------------

static CS_REGISTRY: LazyLock<ShaderRegistry> = LazyLock::new(ShaderRegistry::new);
static PS_REGISTRY: LazyLock<ShaderRegistry> = LazyLock::new(ShaderRegistry::new);
static VS_REGISTRY: LazyLock<ShaderRegistry> = LazyLock::new(ShaderRegistry::new);

static CS_TRACKER: LazyLock<ShaderTracker> = LazyLock::new(ShaderTracker::new);
static PS_TRACKER: LazyLock<ShaderTracker> = LazyLock::new(ShaderTracker::new);
static VS_TRACKER: LazyLock<ShaderTracker> = LazyLock::new(ShaderTracker::new);

// ---------------------------------------------------------------------------
// Context Active Hash Getters
// ---------------------------------------------------------------------------

#[inline(always)]
pub fn get_current_cs_hash(ctx: usize) -> Option<u64> {
    CS_REGISTRY.get_hash(CS_TRACKER.get_active(ctx)?)
}

#[inline(always)]
pub fn get_current_ps_hash(ctx: usize) -> Option<u64> {
    PS_REGISTRY.get_hash(PS_TRACKER.get_active(ctx)?)
}

#[inline(always)]
pub fn get_current_vs_hash(ctx: usize) -> Option<u64> {
    VS_REGISTRY.get_hash(VS_TRACKER.get_active(ctx)?)
}

// ---------------------------------------------------------------------------
// 3. Async Dump Queue
// ---------------------------------------------------------------------------

struct DumpJob {
    stage: &'static str,
    hash: u64,
}

/// Bounded channel — try_send on the hot path means we never block the D3D
/// hook thread. Jobs dropped during a burst are harmless; the bytecode stays
/// in the registry and the file will be written on the next creation event.
const DUMP_QUEUE_DEPTH: usize = 512;

static DUMP_TX: LazyLock<std::sync::mpsc::SyncSender<DumpJob>> = LazyLock::new(|| {
    let (tx, rx) = std::sync::mpsc::sync_channel(DUMP_QUEUE_DEPTH);
    std::thread::Builder::new()
        .name("shader-dump".into())
        .spawn(move || dump_worker(rx))
        .expect("[!] [ShaderDump] Failed to spawn dump thread");
    tx
});

fn dump_worker(rx: std::sync::mpsc::Receiver<DumpJob>) {
    // Session-local dedup: once a hash is in here we never touch the
    // filesystem for it again, not even for a stat call.
    let mut seen: rapidhash::RapidHashSet<u64> = rapidhash::RapidHashSet::default();

    for job in rx {
        // Cheapest check first: already handled this session.
        if seen.contains(&job.hash) {
            continue;
        }

        let registry: &ShaderRegistry = match job.stage {
            "CS" => &CS_REGISTRY,
            "PS" => &PS_REGISTRY,
            "VS" => &VS_REGISTRY,
            _    => continue,
        };

        let bytecode = match registry.get_bytecode(job.hash) {
            Some(b) => b,
            None => continue,
        };

        let dir = Path::new(DUMP_BASE).join(job.stage);
        if let Err(e) = std::fs::create_dir_all(&dir) {
            log!("[!] [ShaderDump] Cannot create dir {:?}: {}", dir, e);
            continue;
        }

        let path = dir.join(format!("{:x}.dxbc", job.hash));

        // Cross-session dedup: one stat per novel hash per process lifetime.
        if !path.exists() {
            match std::fs::write(&path, &bytecode) {
                Ok(_)  => log!("[+] [ShaderDump] {} {:x} → {:?}", job.stage, job.hash, path),
                Err(e) => log!("[!] [ShaderDump] Write failed for {:?}: {}", path, e),
            }
        }

        // Mark seen whether we wrote or it already existed — we never need to
        // touch the filesystem for this hash again this session.
        seen.insert(job.hash);
    }
}

/// Enqueues a dump job. Returns immediately — never blocks the caller.
/// Initialises the worker thread on first call when dump_shaders is true.
fn maybe_dump_shader(stage: &'static str, hash: u64) {
    let should_dump = CONFIG
        .get()
        .map(|c| c.load().dump_shaders)
        .unwrap_or(false);

    if !should_dump {
        return;
    }

    // Non-blocking: if the queue is full we move on; no D3D stall.
    let _ = DUMP_TX.try_send(DumpJob { stage, hash });
}

// ---------------------------------------------------------------------------
// 4. Interceptor Gatekeeper
// ---------------------------------------------------------------------------

#[inline(always)]
fn is_shader_allowed(
    hash: u64,
    get_config_stage: impl FnOnce(&crate::config::Config) -> &crate::config::ShaderConfig,
) -> bool {
    let config_guard = match CONFIG.get() {
        Some(c) => c.load(),
        None => return true,
    };
    !get_config_stage(&config_guard).skip.contains(&ShaderHex(hash))
}

// ---------------------------------------------------------------------------
// Compute Shader Public API
// ---------------------------------------------------------------------------

pub fn register_new_cs(bytecode_ptr: *const u8, len: usize, shader_ptr: usize) {
    if let Some(hash) = CS_REGISTRY.register(bytecode_ptr, len, shader_ptr) {
        maybe_dump_shader("CS", hash);
    }
}

pub fn set_active_cs(ctx: usize, ptr: usize) {
    CS_TRACKER.set_active(&CS_REGISTRY, ctx, ptr);
}

#[inline(always)]
pub fn is_active_cs_allowed(hash: u64) -> bool {
    is_shader_allowed(hash, |cfg| &cfg.compute_shaders)
}

// ---------------------------------------------------------------------------
// Pixel Shader Public API
// ---------------------------------------------------------------------------

pub fn register_new_ps(bytecode_ptr: *const u8, len: usize, shader_ptr: usize) {
    if let Some(hash) = PS_REGISTRY.register(bytecode_ptr, len, shader_ptr) {
        maybe_dump_shader("PS", hash);
    }
}

pub fn set_active_ps(ctx: usize, ptr: usize) {
    PS_TRACKER.set_active(&PS_REGISTRY, ctx, ptr);
}

#[inline(always)]
pub fn is_active_ps_allowed(hash: u64) -> bool {
    is_shader_allowed(hash, |cfg| &cfg.pixel_shaders)
}

// ---------------------------------------------------------------------------
// Vertex Shader Public API
// ---------------------------------------------------------------------------

pub fn register_new_vs(bytecode_ptr: *const u8, len: usize, shader_ptr: usize) {
    if let Some(hash) = VS_REGISTRY.register(bytecode_ptr, len, shader_ptr) {
        maybe_dump_shader("VS", hash);
    }
}

pub fn set_active_vs(ctx: usize, ptr: usize) {
    VS_TRACKER.set_active(&VS_REGISTRY, ctx, ptr);
}

#[inline(always)]
pub fn is_active_vs_allowed(hash: u64) -> bool {
    is_shader_allowed(hash, |cfg| &cfg.vertex_shaders)
}

// ---------------------------------------------------------------------------
// Axum Endpoint Query Functions
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Shaders {
    pub compute: Vec<u64>,
    pub pixel: Vec<u64>,
    pub vertex: Vec<u64>,
}

pub fn get_created_shaders() -> Shaders {
    Shaders {
        compute: CS_REGISTRY.get_all_hashes(),
        pixel: PS_REGISTRY.get_all_hashes(),
        vertex: VS_REGISTRY.get_all_hashes(),
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct StageShaderStats {
    pub compute: RapidHashMap<u64, ShaderStats>,
    pub pixel: RapidHashMap<u64, ShaderStats>,
    pub vertex: RapidHashMap<u64, ShaderStats>,
}

pub fn get_shader_stats() -> StageShaderStats {
    StageShaderStats {
        compute: CS_REGISTRY.get_stats(),
        pixel: PS_REGISTRY.get_stats(),
        vertex: VS_REGISTRY.get_stats(),
    }
}

pub fn get_shader_bytecode(stage: &str, hash: u64) -> Option<Vec<u8>> {
    let registry = match stage {
        "CS" | "compute" => &*CS_REGISTRY,
        "PS" | "pixel"   => &*PS_REGISTRY,
        "VS" | "vertex"  => &*VS_REGISTRY,
        _                => return None,
    };
    registry.get_bytecode(hash)
}

// ---------------------------------------------------------------------------
// 5. HLSL Runtime Compiler
// ---------------------------------------------------------------------------



// ---------------------------------------------------------------------------
// 6. Replacement Storage & Hot Swap Engine
// ---------------------------------------------------------------------------

pub struct ReplacementMap {
    /// Original shader hash → replacement COM pointer address
    map: ArcSwap<RapidHashMap<u64, usize>>,
}

impl ReplacementMap {
    fn new() -> Self {
        Self {
            map: ArcSwap::from_pointee(RapidHashMap::default()),
        }
    }

    #[inline(always)]
    pub fn get(&self, hash: u64) -> Option<*mut c_void> {
        self.map.load().get(&hash).map(|&ptr| ptr as *mut c_void)
    }

    pub fn swap_and_cleanup_cs(&self, new_map: RapidHashMap<u64, usize>) {
        let old_map = self.map.swap(std::sync::Arc::new(new_map));
        for &old_ptr in old_map.values() {
            if old_ptr != 0 {
                unsafe { let _ = ID3D11ComputeShader::from_raw(old_ptr as *mut c_void); }
            }
        }
    }

    pub fn swap_and_cleanup_ps(&self, new_map: RapidHashMap<u64, usize>) {
        let old_map = self.map.swap(std::sync::Arc::new(new_map));
        for &old_ptr in old_map.values() {
            if old_ptr != 0 {
                unsafe { let _ = ID3D11PixelShader::from_raw(old_ptr as *mut c_void); }
            }
        }
    }
}

static CS_REPLACEMENTS: LazyLock<ReplacementMap> = LazyLock::new(ReplacementMap::new);
static PS_REPLACEMENTS: LazyLock<ReplacementMap> = LazyLock::new(ReplacementMap::new);

// ---------------------------------------------------------------------------
// Helper: find ./IndirectX_shaders/Replacements/<stage>/<hash>.<any ext>
// ---------------------------------------------------------------------------

fn find_replacement_file(dir: &Path, hash: u64) -> Option<std::path::PathBuf> {
    let prefix = format!("{:x}.", hash);
    std::fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with(&prefix))
                .unwrap_or(false)
        })
}

// ---------------------------------------------------------------------------
// Reload API
// ---------------------------------------------------------------------------

pub fn reload_replacement_shaders() {
    let device_ptr = match get_device() {
        Some(ptr) => ptr,
        None => {
            log!("[!] [ShaderManager] Cannot reload replacements: ID3D11Device is null");
            return;
        }
    };

    let config_guard = match CONFIG.get() {
        Some(c) => c.load(),
        None => return,
    };

    // 1. Process Compute Shaders
    let mut new_cs_map = RapidHashMap::default();
    let cs_dir = Path::new(REPLACE_BASE).join("CS");

    for shader_hex in &config_guard.compute_shaders.replace {
        let hash = shader_hex.0;

        let file_path = match find_replacement_file(&cs_dir, hash) {
            Some(p) => p,
            None => {
                log!("[!] [ShaderManager] CS replacement missing for {:x}", hash);
                continue;
            }
        };

        let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let bytecode = match ext {
            "hlsl" => hlsl_to_dxbc(&file_path, b"cs_5_0\0"),
            "shdr" => {
                // shdr_to_dxbc splices the raw SHDR chunk from the file into
                // the original shader's DXBC container, preserving all other
                // chunks (RDEF, ISGN, OSGN, …). We therefore need the
                // original bytecode out of the registry.
                match CS_REGISTRY.get_bytecode(hash) {
                    Some(original) => shdr_to_dxbc(original, &file_path),
                    None => {
                        log!("[!] [ShaderManager] CS {:x}: original bytecode not in registry (shader never created?)", hash);
                        continue;
                    }
                }
            }
            "asm"  => { /* TODO: assemble DXBC ASM */ continue; }
            "txt"  => { /* TODO: treat as DXBC ASM text */ continue; }
            "dxbc" => { /* TODO: load raw DXBC directly */ continue; }
            other  => {
                log!("[!] [ShaderManager] CS {:x}: unknown replacement extension '{}'", hash, other);
                continue;
            }
        };

        if let Some(bytecode) = bytecode {
            let new_ptr = unsafe { create_sub_cs(device_ptr, &bytecode) };
            if !new_ptr.is_null() {
                new_cs_map.insert(hash, new_ptr as usize);
                log!("[+] [ShaderManager] Replaced CS {:x} ({})", hash, ext);
            } else {
                log!("[!] [ShaderManager] CS {:x}: object creation returned null", hash);
            }
        } else {
            log!("[!] [ShaderManager] CS {:x}: compilation/patching failed for '{}'", hash, ext);
        }
    }

    // 2. Process Pixel Shaders
    let mut new_ps_map = RapidHashMap::default();
    let ps_dir = Path::new(REPLACE_BASE).join("PS");

    for shader_hex in &config_guard.pixel_shaders.replace {
        let hash = shader_hex.0;

        let file_path = match find_replacement_file(&ps_dir, hash) {
            Some(p) => p,
            None => {
                log!("[!] [ShaderManager] PS replacement missing for {:x}", hash);
                continue;
            }
        };

        let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let bytecode = match ext {
            "hlsl" => hlsl_to_dxbc(&file_path, b"ps_5_0\0"),
            "shdr" => {
                // Same rationale as CS above — original container is needed
                // to preserve all non-SHDR chunks during the splice.
                match PS_REGISTRY.get_bytecode(hash) {
                    Some(original) => shdr_to_dxbc(original, &file_path),
                    None => {
                        log!("[!] [ShaderManager] PS {:x}: original bytecode not in registry (shader never created?)", hash);
                        continue;
                    }
                }
            }
            "asm"  => { /* TODO: assemble DXBC ASM */ continue; }
            "txt"  => { /* TODO: treat as DXBC ASM text */ continue; }
            "dxbc" => { /* TODO: load raw DXBC directly */ continue; }
            other  => {
                log!("[!] [ShaderManager] PS {:x}: unknown replacement extension '{}'", hash, other);
                continue;
            }
        };

        if let Some(bytecode) = bytecode {
            let new_ptr = unsafe { create_sub_ps(device_ptr, &bytecode) };
            if !new_ptr.is_null() {
                new_ps_map.insert(hash, new_ptr as usize);
                log!("[+] [ShaderManager] Replaced PS {:x} ({})", hash, ext);
            } else {
                log!("[!] [ShaderManager] PS {:x}: object creation returned null", hash);
            }
        } else {
            log!("[!] [ShaderManager] PS {:x}: compilation/patching failed for '{}'", hash, ext);
        }
    }

    // Atomic swap + GPU object cleanup
    CS_REPLACEMENTS.swap_and_cleanup_cs(new_cs_map);
    PS_REPLACEMENTS.swap_and_cleanup_ps(new_ps_map);
}

// ---------------------------------------------------------------------------
// Public Replacement Getters for Interceptors
// ---------------------------------------------------------------------------

#[inline(always)]
pub fn get_cs_replacement(ctx: usize) -> Option<*mut c_void> {
    CS_REPLACEMENTS.get(get_current_cs_hash(ctx)?)
}

#[inline(always)]
pub fn get_ps_replacement(ctx: usize) -> Option<*mut c_void> {
    PS_REPLACEMENTS.get(get_current_ps_hash(ctx)?)
}