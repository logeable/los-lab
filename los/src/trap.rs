use ansi_rgb::{yellow, yellow_green, Foreground};
use core::arch::global_asm;
use riscv::register::{scause, sepc, sstatus, stval, stvec};

use crate::{
    batch::{run_next_app, APP_LOADER},
    println, syscall,
};

global_asm!(include_str!("trap.asm"));

pub fn init() {
    extern "C" {
        fn s_trap_enter();
    }

    unsafe {
        stvec::write(s_trap_enter as usize, stvec::TrapMode::Direct);
    }
}

#[no_mangle]
pub fn process_trap(ctx: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read();
    let stval = stval::read();

    match scause.cause() {
        scause::Trap::Interrupt(intr) => match intr {
            scause::Interrupt::UserSoft => todo!(),
            scause::Interrupt::VirtualSupervisorSoft => todo!(),
            scause::Interrupt::SupervisorSoft => todo!(),
            scause::Interrupt::UserTimer => todo!(),
            scause::Interrupt::VirtualSupervisorTimer => todo!(),
            scause::Interrupt::SupervisorTimer => todo!(),
            scause::Interrupt::UserExternal => todo!(),
            scause::Interrupt::VirtualSupervisorExternal => todo!(),
            scause::Interrupt::SupervisorExternal => todo!(),
            scause::Interrupt::Unknown => todo!(),
        },
        scause::Trap::Exception(ex) => match ex {
            scause::Exception::InstructionMisaligned => todo!(),
            scause::Exception::InstructionFault => todo!(),
            scause::Exception::IllegalInstruction => {
                println!(
                    "{}",
                    format_args!("[TRAP] illegal instruction: {:x}", stval).fg(yellow())
                );
                run_next_app();
            }
            scause::Exception::Breakpoint => todo!(),
            scause::Exception::LoadFault => todo!(),
            scause::Exception::StoreMisaligned => todo!(),
            scause::Exception::StoreFault => todo!(),
            scause::Exception::UserEnvCall => {
                ctx.regs[10] =
                    syscall::syscall(ctx.regs[17], ctx.regs[10], ctx.regs[11], ctx.regs[12]);
                ctx.sepc += 4;
            }
            scause::Exception::VirtualSupervisorEnvCall => todo!(),
            scause::Exception::InstructionPageFault => todo!(),
            scause::Exception::LoadPageFault => todo!(),
            scause::Exception::StorePageFault => todo!(),
            scause::Exception::InstructionGuestPageFault => todo!(),
            scause::Exception::LoadGuestPageFault => todo!(),
            scause::Exception::VirtualInstruction => todo!(),
            scause::Exception::StoreGuestPageFault => todo!(),
            scause::Exception::Unknown => todo!(),
        },
    }

    ctx
}

#[repr(C)]
pub struct TrapContext {
    pub regs: [usize; 32],
    pub sstatus: sstatus::Sstatus,
    pub sepc: usize,
}

impl TrapContext {
    pub fn set_sp(&mut self, sp: usize) {
        self.regs[2] = sp;
    }

    pub fn init(entry: usize, sp: usize) -> Self {
        let mut ctx = TrapContext {
            regs: [0; 32],
            sstatus: sstatus::read(),
            sepc: entry,
        };
        ctx.set_sp(sp);

        ctx
    }
}

pub fn return_to_user(ctx_ptr: *const TrapContext) -> ! {
    extern "C" {
        fn s_trap_return(ctx_ptr: usize) -> !;
    }

    unsafe {
        s_trap_return(ctx_ptr as usize);
    }
}
