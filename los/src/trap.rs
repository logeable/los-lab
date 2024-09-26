use crate::{println, syscall, task, timer};
use riscv::register::{scause, sepc, sie, sstatus, stval, stvec};

pub fn init() {
    extern "C" {
        fn _s_trap_enter();
    }

    unsafe {
        stvec::write(_s_trap_enter as usize, stvec::TrapMode::Direct);
    }

    unsafe { sie::set_stimer() };
}

#[no_mangle]
pub fn process_trap(ctx: &mut TrapContext) {
    let scause = scause::read();
    let stval = stval::read();

    match scause.cause() {
        scause::Trap::Interrupt(intr) => match intr {
            scause::Interrupt::SupervisorTimer => {
                timer::set_next_trigger();
                task::suspend_current_task_and_schedule()
            }
            _ => {
                unimplemented!("Interrupt handler not implemented: {:?}", intr);
            }
        },
        scause::Trap::Exception(ex) => match ex {
            scause::Exception::IllegalInstruction => {
                println!(
                    "[TRAP] illegal instruction: {:#x} {:#x} {:?} sie: {} spie: {}, {:?}",
                    stval,
                    sepc::read(),
                    sstatus::read().spp(),
                    sstatus::read().sie(),
                    sstatus::read().spie(),
                    ctx,
                );

                task::exit_current_task_and_schedule()
            }
            scause::Exception::UserEnvCall => {
                ctx.regs[10] =
                    syscall::syscall(ctx.regs[17], ctx.regs[10], ctx.regs[11], ctx.regs[12]);
                ctx.sepc += 4;
            }
            scause::Exception::StoreFault => {
                println!(
                    "[TRAP] store fault: {:#x} {:#x} {:?}",
                    stval,
                    sepc::read(),
                    ctx
                );
                task::exit_current_task_and_schedule()
            }
            scause::Exception::StorePageFault => {
                println!(
                    "[TRAP] store page fault: {:#x} {:#x} {:?}",
                    stval,
                    sepc::read(),
                    ctx
                );
                task::exit_current_task_and_schedule()
            }
            scause::Exception::InstructionFault => {
                println!(
                    "[TRAP] instruction fault: {:#x} {:#x} {:?}",
                    stval,
                    sepc::read(),
                    ctx
                );
                task::exit_current_task_and_schedule()
            }
            scause::Exception::InstructionPageFault => {
                println!(
                    "[TRAP] instruction page fault: {:#x} {:#x} {:?}",
                    stval,
                    sepc::read(),
                    ctx
                );
                task::exit_current_task_and_schedule()
            }
            _ => {
                unimplemented!("Exception handler not implemented: {:?}", ex);
            }
        },
    }
}

#[repr(C)]
#[derive(Debug)]
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
