use crate::{
    mm::{self},
    println, syscall,
    task::processor,
    timer,
};
use core::arch::asm;
use riscv::register::{scause, sepc, sie, sstatus, stval, stvec};

pub fn init() {
    set_stvec_to_user_trap();
    init_timer();
}

fn set_stvec_to_user_trap() {
    unsafe {
        stvec::write(mm::trampoline_va().into(), stvec::TrapMode::Direct);
    }
}

fn set_stvec_to_kernel_trap() {
    unsafe {
        stvec::write(kernel_trap_entry as usize, stvec::TrapMode::Direct);
    }
}

#[no_mangle]
fn kernel_trap_entry() {
    panic!("trap in kernel mode")
}

pub fn user_trap_return_va() -> usize {
    extern "C" {
        fn _s_trap_enter();
        fn _s_trap_return();
    }

    (mm::trampoline_va() + (_s_trap_return as usize - _s_trap_enter as usize)).into()
}

fn init_timer() {
    unsafe { sie::set_stimer() };
}

#[no_mangle]
pub fn process_trap() -> ! {
    set_stvec_to_kernel_trap();

    let scause = scause::read();
    let stval = stval::read();

    let trap_context = unsafe {
        &mut *processor::get_current_task_trap_context()
            .expect("current task trap context must exist")
    };

    match scause.cause() {
        scause::Trap::Interrupt(intr) => match intr {
            scause::Interrupt::SupervisorTimer => {
                timer::set_next_trigger();
                processor::suspend_current_task_and_schedule()
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
                    trap_context,
                );

                processor::exit_current_task_and_schedule(-1)
            }
            scause::Exception::UserEnvCall => {
                trap_context.regs[10] = syscall::syscall(
                    trap_context.regs[17],
                    trap_context.regs[10],
                    trap_context.regs[11],
                    trap_context.regs[12],
                );
                trap_context.sepc += 4;
            }
            scause::Exception::StoreFault => {
                println!(
                    "[TRAP] store fault: {:#x} {:#x} {:?}",
                    stval,
                    sepc::read(),
                    trap_context
                );
                processor::exit_current_task_and_schedule(-1)
            }
            scause::Exception::StorePageFault => {
                println!(
                    "[TRAP] store page fault: {:#x} {:#x} {:?}",
                    stval,
                    sepc::read(),
                    trap_context
                );
                processor::exit_current_task_and_schedule(-1)
            }
            scause::Exception::InstructionFault => {
                println!(
                    "[TRAP] instruction fault: {:#x} {:#x} {:?}",
                    stval,
                    sepc::read(),
                    trap_context
                );
                processor::exit_current_task_and_schedule(-1)
            }
            scause::Exception::InstructionPageFault => {
                println!(
                    "[TRAP] instruction page fault: {:#x} {:#x} {:?}",
                    stval,
                    sepc::read(),
                    trap_context
                );
                processor::exit_current_task_and_schedule(-1)
            }
            scause::Exception::LoadPageFault => {
                println!(
                    "[TRAP] load page fault: {:#x} {:#x} {:?}",
                    stval,
                    sepc::read(),
                    trap_context
                );
                processor::exit_current_task_and_schedule(-1)
            }
            _ => {
                unimplemented!("Exception handler not implemented: {:?}", ex);
            }
        },
    }

    trap_return();
}

#[no_mangle]
pub fn trap_return() -> ! {
    set_stvec_to_user_trap();

    let return_va = user_trap_return_va();

    let trap_context_va = mm::trap_context_va();
    let trap_context_ptr: usize = trap_context_va.into();

    let app_satp = processor::get_current_task_satp();

    unsafe {
        asm!(
            "fence.i",
            "jr {return_va}",
            return_va = in(reg) return_va,
            in("a0") trap_context_ptr,
            in("a1") app_satp,
            options(noreturn)
        );
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct TrapContext {
    pub regs: [usize; 32],
    pub sstatus: sstatus::Sstatus,
    pub sepc: usize,
    pub kernel_satp: usize,
    pub kernel_sp: usize,
    pub trap_handler: usize,
}

impl TrapContext {
    pub fn set_user_sp(&mut self, sp: usize) {
        self.regs[2] = sp;
    }

    pub fn set_entry(&mut self, entry: usize) {
        self.sepc = entry;
    }

    pub fn init(entry: usize, user_sp: usize, kernel_sp: usize) -> Self {
        extern "C" {
            fn process_trap();
        }

        let mut ctx = TrapContext {
            regs: [0; 32],
            sstatus: sstatus::read(),
            sepc: entry,
            kernel_satp: mm::kernel_satp(),
            kernel_sp,
            trap_handler: process_trap as usize,
        };
        ctx.set_user_sp(user_sp);

        ctx
    }
}
