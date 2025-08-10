//! x64架构的CPU操作实现
use crate::hal_fn::cpu::{__HalImpl, __HalTrait};
use raw_cpuid::CpuId;
use trapframe::UserContext;

impl __HalTrait for __HalImpl {
    fn cpu_id() -> u8 {
        CpuId::new()
            .get_feature_info()
            .unwrap()
            .initial_local_apic_id()
    }

    fn cpu_frequency() -> u16 {
        static CPU_FREQ_MHZ: spin::Once<u16> = spin::Once::new();
        *CPU_FREQ_MHZ.call_once(|| {
            const DEFAULT: u16 = 4000;
            CpuId::new()
                .get_processor_frequency_info()
                .map(|info| info.processor_base_frequency())
                .unwrap_or(DEFAULT)
                .max(DEFAULT)
        })
    }

    fn reset() -> ! {
        unimplemented!()
    }
}

/// struct mcontext
#[repr(C)]
#[derive(Clone, Debug)]
pub struct MachineContext {
    // gregs
    pub r8: usize,
    pub r9: usize,
    pub r10: usize,
    pub r11: usize,
    pub r12: usize,
    pub r13: usize,
    pub r14: usize,
    pub r15: usize,
    pub rdi: usize,
    pub rsi: usize,
    pub rbp: usize,
    pub rbx: usize,
    pub rdx: usize,
    pub rax: usize,
    pub rcx: usize,
    pub rsp: usize,
    pub rip: usize,
    pub eflags: usize,
    pub cs: u16,
    pub gs: u16,
    pub fs: u16,
    pub _pad: u16,
    pub err: usize,
    pub trapno: usize,
    pub oldmask: usize,
    pub cr2: usize,
    // fpregs
    // TODO
    pub fpstate: usize,
    // reserved
    pub _reserved1: [usize; 8],
}

impl MachineContext {
    pub fn from_tf(tf: &UserContext) -> Self {
        Self {
            r8: tf.general.r8,
            r9: tf.general.r9,
            r10: tf.general.r10,
            r11: tf.general.r11,
            r12: tf.general.r12,
            r13: tf.general.r13,
            r14: tf.general.r14,
            r15: tf.general.r15,
            rdi: tf.general.rdi,
            rsi: tf.general.rsi,
            rbp: tf.general.rbp,
            rbx: tf.general.rbx,
            rdx: tf.general.rdx,
            rax: tf.general.rax,
            rcx: tf.general.rcx,
            rsp: tf.general.rsp,
            rip: tf.general.rip,
            eflags: tf.general.rflags,
            cs: 0,
            gs: 0,
            fs: 0,
            _pad: 0,
            err: tf.error_code,
            trapno: tf.trap_num,
            oldmask: 0,
            cr2: 0,
            fpstate: 0,
            _reserved1: [0; 8],
        }
    }
}
