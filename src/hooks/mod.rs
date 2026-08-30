pub mod device;
pub mod swapchain;
pub mod context;
pub mod dxgi_factory;

use crate::log;
use std::sync::{Mutex, LazyLock};
use vmt_hook::VTableHook;
use std::ffi::c_void;

struct HookWrapper(VTableHook<*mut c_void>);
unsafe impl Sync for HookWrapper {}
unsafe impl Send for HookWrapper {}

#[macro_export]
macro_rules! make_hook_map {
    
    
    ( $( ($index:expr, $module:ident) ),* $(,)? ) => {
        [
            $(
                (
                    $index as usize,
                    $module::set_orig_func as fn(usize) -> (),
                    $module::hooked_func as *const () as usize
                ),
            )*
        ]
    };
}

static HOOKS_STORAGE: LazyLock<Mutex<Vec<HookWrapper>>> = LazyLock::new(|| {
    Mutex::new(Vec::new())
});

pub fn register_hook(hook: VTableHook<*mut c_void>) {
    HOOKS_STORAGE.lock().unwrap().push(HookWrapper(hook));
}

fn install_hooks(com: *mut c_void, hook_map:&[(usize, fn(usize), usize)]){
    unsafe {
        let hook_slot = VTableHook::new(com);
        for hook in hook_map {
            let (index, set_orig, target) = *hook;
            set_orig(hook_slot.get_original_method(index));
            hook_slot.replace_method(index, target);
            log!("replaced method for {}", index);
        }
        register_hook(hook_slot);
    }
}

