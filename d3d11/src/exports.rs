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

            #[cfg(target_arch = "x86")]
            #[unsafe(no_mangle)]
            #[unsafe(naked)]
            pub unsafe extern "system" fn $name() -> ! {
                core::arch::naked_asm!(
                    "mov eax, dword ptr [{target}]",
                    "jmp eax",
                    target = sym [<$name _ORIG_PTR>],
                );
            }

            #[cfg(target_arch = "x86_64")]
            #[unsafe(no_mangle)]
            #[unsafe(naked)]
            pub unsafe extern "system" fn $name() -> ! {
                core::arch::naked_asm!(
                    "mov rax, qword ptr [rip + {target}]",
                    "jmp rax",
                    target = sym [<$name _ORIG_PTR>],
                );
            }
        }
    };
}

naked_trampoline!(D3D11CoreCreateDevice);
naked_trampoline!(D3D11On12CreateDevice);
naked_trampoline!(D3D11CreateDevice);
naked_trampoline!(D3D11CreateDeviceAndSwapChain);
