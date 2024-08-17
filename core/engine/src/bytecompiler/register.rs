use std::mem::forget;

bitflags::bitflags! {
    #[derive(Debug, Default, Clone, Copy)]
    struct RegisterFlags: u8 {
        const USED       = 0b0000_0001;
        const PERSISTENT = 0b0000_0010;
    }
}

impl RegisterFlags {
    fn is_used(self) -> bool {
        self.contains(Self::USED)
    }
    fn is_persistent(self) -> bool {
        self.contains(Self::PERSISTENT)
    }
}

#[derive(Debug, Default)]
pub(crate) struct Register {
    flags: RegisterFlags,
}

#[derive(Debug)]
pub(crate) struct Reg {
    index: u32,
    flags: RegisterFlags,
}

impl Reg {
    pub(crate) fn index(&self) -> u32 {
        self.index
    }
}

impl Drop for Reg {
    fn drop(&mut self) {
        if !self.flags.is_persistent() {
            unreachable!("forgot to deallocate a register! Or a panic happend which caused Reg's drop to be called after the panic!")
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct RegisterAllocator {
    registers: Vec<Register>,
}

impl RegisterAllocator {
    pub(crate) fn alloc(&mut self) -> Reg {
        for (i, register) in self.registers.iter_mut().enumerate() {
            if register.flags.is_used() {
                continue;
            }

            assert!(!register.flags.is_persistent());

            register.flags |= RegisterFlags::USED;
            return Reg {
                index: i as u32,
                flags: register.flags,
            };
        }

        let flags = RegisterFlags::USED;

        let index = self.registers.len() as u32;
        self.registers.push(Register { flags });

        Reg { index, flags }
    }

    pub(crate) fn alloc_persistent(&mut self) -> Reg {
        let mut reg = self.alloc();

        let index = reg.index();

        let register = &mut self.registers[index as usize];

        register.flags |= RegisterFlags::PERSISTENT;

        reg.flags = register.flags;
        reg
    }

    pub(crate) fn dealloc(&mut self, reg: Reg) {
        assert!(
            !reg.flags.is_persistent(),
            "Trying to deallocate a persistent register"
        );

        let register = &mut self.registers[reg.index as usize];

        assert!(
            register.flags.is_used(),
            "Cannot deallocate unused variable"
        );
        register.flags.set(RegisterFlags::USED, false);

        // NOTE: We should not drop it since, dropping it used to detect bugs.
        forget(reg);
    }

    pub(crate) fn finish(self) -> u32 {
        for register in &self.registers {
            debug_assert!(
                !register.flags.is_used()
                    || (register.flags.is_used() && register.flags.is_persistent())
            );
        }

        self.registers.len() as u32
    }
}
