use nix::libc::{Elf32_Ehdr, Elf32_Phdr};
use std::arch::asm;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Elf32AuxV {
    pub a_type: u32,
    pub a_un: Elf32AuxVBindgenTy1,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union Elf32AuxVBindgenTy1 {
    pub a_val: u32,
}

pub const AT_NULL: u32 = 0;
pub const AT_PHDR: u32 = 3;
pub const AT_ENTRY: u32 = 9;

extern "C" {
    static environ: *mut *mut u8;
}

pub fn exec_run(base_address: usize, entry_point: usize , env_address: usize) {
    let ehdr = unsafe { &*(base_address as *const u8 as *const Elf32_Ehdr) };
    let phdr_table = unsafe {
        &*(base_address as *const u8 as *const Elf32_Phdr)
            .add(ehdr.e_phoff as usize / std::mem::size_of::<Elf32_Phdr>())
    };
    let mut auxv = unsafe {
        let mut env = environ;
        // skip environment variables
        while !(*env).is_null() {
            env = env.offset(1);
        }
        env = env.offset(1);
        &mut *(env as *mut u8 as *mut Elf32AuxV)
    };

    while auxv.a_type != AT_NULL {
        match auxv.a_type {
            AT_PHDR => auxv.a_un.a_val = phdr_table as *const Elf32_Phdr as u32,
            AT_ENTRY => auxv.a_un.a_val = ehdr.e_entry,
            _ => {}
        }
        auxv = unsafe { &mut *(auxv as *mut Elf32AuxV).offset(1) };
    }

    unsafe {
        asm!(
            "mov esp, {0}
            xor ebx, ebx
            xor ecx, ecx
            xor edx, edx
            xor ebp, ebp
            xor esi, esi
            xor edi, edi
            jmp {1}",
            in(reg) env_address,
            in(reg) entry_point,
        );
    }
}