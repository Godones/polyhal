use kprobe::{kprobe_handler_from_break, PtRegs};
use polyhal::trapframe::TrapFrame;

use crate::kprobe::KPROBE_MANAGER;

#[derive(Debug)]
pub struct EBreak;

impl EBreak {
    pub fn handle(frame: &mut TrapFrame) -> Option<()> {
        Self::kprobe_handler(frame)
    }
    fn kprobe_handler(frame: &mut TrapFrame) -> Option<()> {
        let mut manager = KPROBE_MANAGER.lock();
        let mut pt_regs = pt_regs_from_trapframe(frame);
        let res = kprobe_handler_from_break(&mut manager, &mut pt_regs);
        pt_regs_to_trapframe(frame, &pt_regs);
        res
    }
}

pub fn pt_regs_from_trapframe(frame: &TrapFrame) -> PtRegs {
    #[cfg(target_arch = "x86_64")]
    {
        PtRegs {
            r15: frame.r15,
            r14: frame.r14,
            r13: frame.r13,
            r12: frame.r12,
            rbp: frame.rbp,
            rbx: frame.rbx,
            r11: frame.r11,
            r10: frame.r10,
            r9: frame.r9,
            r8: frame.r8,
            rax: frame.rax,
            rcx: frame.rcx,
            rdx: frame.rdx,
            rsi: frame.rsi,
            rdi: frame.rdi,
            orig_rax: frame.error_code,
            rip: frame.rip,
            cs: frame.cs,
            rflags: frame.rflags,
            rsp: frame.rsp,
            ss: frame.ss,
        }
    }
    #[cfg(target_arch = "riscv64")]
    {
        PtRegs {
            epc: frame.sepc,
            ra: frame.x[1],
            sp: frame.x[2],
            gp: frame.x[3],
            tp: frame.x[4],
            t0: frame.x[5],
            t1: frame.x[6],
            t2: frame.x[7],
            s0: frame.x[8],
            s1: frame.x[9],
            a0: frame.x[10],
            a1: frame.x[11],
            a2: frame.x[12],
            a3: frame.x[13],
            a4: frame.x[14],
            a5: frame.x[15],
            a6: frame.x[16],
            a7: frame.x[17],
            s2: frame.x[18],
            s3: frame.x[19],
            s4: frame.x[20],
            s5: frame.x[21],
            s6: frame.x[22],
            s7: frame.x[23],
            s8: frame.x[24],
            s9: frame.x[25],
            s10: frame.x[26],
            s11: frame.x[27],
            t3: frame.x[28],
            t4: frame.x[29],
            t5: frame.x[30],
            t6: frame.x[31],
            status: 0,
            badaddr: 0,
            cause: 0,
            orig_a0: 0,
        }
    }

    #[cfg(target_arch = "loongarch64")]
    {
        PtRegs {
            regs: frame.regs,
            orig_a0: 0,
            csr_era: frame.era,
            csr_badvaddr: 0,
            csr_crmd: 0,
            csr_prmd: frame.prmd,
            csr_euen: 0,
            csr_ecfg: 0,
            csr_estat: 0,
        }
    }
}

// update pt_regs from trapframe
pub fn pt_regs_to_trapframe(frame: &mut TrapFrame, pt_regs: &PtRegs) {
    #[cfg(target_arch = "x86_64")]
    {
        frame.r15 = pt_regs.r15;
        frame.r14 = pt_regs.r14;
        frame.r13 = pt_regs.r13;
        frame.r12 = pt_regs.r12;
        frame.rbp = pt_regs.rbp;
        frame.rbx = pt_regs.rbx;
        frame.r11 = pt_regs.r11;
        frame.r10 = pt_regs.r10;
        frame.r9 = pt_regs.r9;
        frame.r8 = pt_regs.r8;
        frame.rax = pt_regs.rax;
        frame.rcx = pt_regs.rcx;
        frame.rdx = pt_regs.rdx;
        frame.rsi = pt_regs.rsi;
        frame.rdi = pt_regs.rdi;
        frame.error_code = pt_regs.orig_rax;
        frame.rip = pt_regs.rip;
        frame.cs = pt_regs.cs;
        frame.rflags = pt_regs.rflags;
        frame.rsp = pt_regs.rsp;
        frame.ss = pt_regs.ss;
    }
    #[cfg(target_arch = "riscv64")]
    {
        frame.sepc = pt_regs.epc;
        frame.x[1] = pt_regs.ra;
        frame.x[2] = pt_regs.sp;
        frame.x[3] = pt_regs.gp;
        frame.x[4] = pt_regs.tp;
        frame.x[5] = pt_regs.t0;
        frame.x[6] = pt_regs.t1;
        frame.x[7] = pt_regs.t2;
        frame.x[8] = pt_regs.s0;
        frame.x[9] = pt_regs.s1;
        frame.x[10] = pt_regs.a0;
        frame.x[11] = pt_regs.a1;
        frame.x[12] = pt_regs.a2;
        frame.x[13] = pt_regs.a3;
        frame.x[14] = pt_regs.a4;
        frame.x[15] = pt_regs.a5;
        frame.x[16] = pt_regs.a6;
        frame.x[17] = pt_regs.a7;
        frame.x[18] = pt_regs.s2;
        frame.x[19] = pt_regs.s3;
        frame.x[20] = pt_regs.s4;
        frame.x[21] = pt_regs.s5;
        frame.x[22] = pt_regs.s6;
        frame.x[23] = pt_regs.s7;
        frame.x[24] = pt_regs.s8;
        frame.x[25] = pt_regs.s9;
        frame.x[26] = pt_regs.s10;
        frame.x[27] = pt_regs.s11;
        frame.x[28] = pt_regs.t3;
        frame.x[29] = pt_regs.t4;
        frame.x[30] = pt_regs.t5;
        frame.x[31] = pt_regs.t6;
    }

    #[cfg(target_arch = "loongarch64")]
    {
        frame.regs = pt_regs.regs;
        frame.era = pt_regs.csr_era;
        frame.prmd = pt_regs.csr_prmd;
        // other CSR fields are not used in this context
    }
}
