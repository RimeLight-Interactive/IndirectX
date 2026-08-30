use paste::paste;
use std::arch::naked_asm;

macro_rules! naked_trampoline {
    ($name:ident) => {
        paste! {
            #[allow(non_upper_case_globals)]
            static mut [<$name _ORIG_PTR>]: usize = 0;

            #[allow(dead_code)]
            #[inline(always)]
            pub unsafe fn [<set_ $name _orig>](ptr: usize) {
                [<$name _ORIG_PTR>] = ptr;
            }

            #[unsafe(no_mangle)]
            #[unsafe(naked)]
            pub unsafe extern "system" fn $name() -> ! {
                naked_asm!(
                    "mov rax, qword ptr [rip + {target}]",
                    "jmp rax",
                    target = sym [<$name _ORIG_PTR>],
                );
            }
        }
    };
}

naked_trampoline!(CreateDXGIFactory1);
naked_trampoline!(CreateDXGIFactory2);
naked_trampoline!(DXGIDeclareAdapterRemovalSupport);
naked_trampoline!(DXGIGetDebugInterface1);
naked_trampoline!(CreateDXGIFactory);

/*
CreateDXGIFactory
      10   0xa0b0  CreateDXGIFactory1
      11   0x9fd0  CreateDXGIFactory2
      16   0x9d10  DXGIDeclareAdapterRemovalSupport
      17   0x9df0  DXGIGetDebugInterface1
*/