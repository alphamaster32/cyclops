use crate::mmu::Mmu;
use std::fmt;

/// 32-bit RISC-V registers 
/// These are the ABI names from chapter 25 of ISA specification
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Register {
    Zero = 0,
    Ra,
    Sp,
    Gp,
    Tp,
    T0,
    T1,
    T2,
    S0,
    S1,
    A0,
    A1,
    A2,
    A3,
    A4,
    A5,
    A6,
    A7,
    S2,
    S3,
    S4,
    S5,
    S6,
    S7,
    S8,
    S9,
    S10,
    S11,
    T3,
    T4,
    T5,
    T6,
    Pc,
}

impl From<u32> for Register {
    fn from(val: u32) -> Self {
        assert!(val < 33);
        unsafe {
            // Avoid lenghty match statement
            core::ptr::read_unaligned(&(val as usize) as
                                      *const usize as *const Register)
        }
    }
}

/// An R-type instruction
#[derive(Debug)]
pub struct Rtype {
    funct7: u32,
    rs2:    Register,
    rs1:    Register,
    funct3: u32,
    rd:     Register,
}

impl From<u32> for Rtype {
    fn from(inst: u32) -> Self {
        Rtype {
            funct7: (inst >> 25) & 0b1111111,
            rs2:    Register::from((inst >> 20) & 0b11111),
            rs1:    Register::from((inst >> 15) & 0b11111),
            funct3: (inst >> 12) & 0b111,
            rd:     Register::from((inst >>  7) & 0b11111),
        }
    }
}

/// An S-type instruction
#[derive(Debug)]
pub struct Stype {
    imm:    i32,
    rs2:    Register,
    rs1:    Register,
    funct3: u32,
}

impl From<u32> for Stype {
    fn from(inst: u32) -> Self {
        let imm115 = (inst >> 25) & 0b1111111;
        let imm40  = (inst >>  7) & 0b11111;

        let imm = (imm115 << 5) | imm40;
        let imm = ((imm as i32) << 20) >> 20;

        Stype {
            imm:    imm,
            rs2:    Register::from((inst >> 20) & 0b11111),
            rs1:    Register::from((inst >> 15) & 0b11111),
            funct3: (inst >> 12) & 0b111,
        }
    }
}

/// A J-type instruction
#[derive(Debug)]
pub struct Jtype {
    imm: i32,
    rd:  Register,
}

impl From<u32> for Jtype {
    fn from(inst: u32) -> Self {
        let imm20   = (inst >> 31) & 1;
        let imm101  = (inst >> 21) & 0b1111111111;
        let imm11   = (inst >> 20) & 1;
        let imm1912 = (inst >> 12) & 0b11111111;

        let imm = (imm20 << 20) | (imm1912 << 12) | (imm11 << 11) |
            (imm101 << 1);
        let imm = ((imm as i32) << 11) >> 11;

        Jtype {
            imm: imm,
            rd:  Register::from((inst >> 7) & 0b11111),
        }
    }
}

/// A B-type instruction
#[derive(Debug)]
pub struct Btype {
    imm:    i32,
    rs2:    Register,
    rs1:    Register,
    funct3: u32,
}

impl From<u32> for Btype {
    fn from(inst: u32) -> Self {
        let imm12  = (inst >> 31) & 1;
        let imm105 = (inst >> 25) & 0b111111;
        let imm41  = (inst >>  8) & 0b1111;
        let imm11  = (inst >>  7) & 1;

        let imm = (imm12 << 12) | (imm11 << 11) |(imm105 << 5) | (imm41 << 1);
        let imm = ((imm as i32) << 19) >> 19;

        Btype {
            imm:    imm,
            rs2:    Register::from((inst >> 20) & 0b11111),
            rs1:    Register::from((inst >> 15) & 0b11111),
            funct3: (inst >> 12) & 0b111,
        }
    }
}

/// An I-type instruction
#[derive(Debug)]
pub struct Itype {
    imm:    i32,
    rs1:    Register,
    funct3: u32,
    rd:     Register,
}

impl From<u32> for Itype {
    fn from(inst: u32) -> Self {
        let imm = (inst as i32) >> 20;
        Itype {
            imm:    imm,
            rs1:    Register::from((inst >> 15) & 0b11111),
            funct3: (inst >> 12) & 0b111,
            rd:     Register::from((inst >>  7) & 0b11111),
        }
    }
}

/// An U-type instructions
#[derive(Debug)]
pub struct Utype {
    imm: i32,
    rd:  Register,
}

impl From<u32> for Utype {
    fn from(inst: u32) -> Self {
        Utype {
            imm: (inst & !0xfff) as i32,
            rd:  Register::from((inst >> 7) & 0b11111),
        }
    }
}

/// An enum representing the instruction type to be embedded into the `Inst`
/// struct
#[derive(Debug)]
pub enum Type {
    None,
    U(Utype),
    I(Itype),
    B(Btype),
    J(Jtype),
    S(Stype),
    R(Rtype),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Opcode {
    None,
    LUI,
    AUIPC,
    ADDI,
    SUB,
    JAL,
}

impl From<u32> for Opcode {
    fn from(val: u32) -> Self {
        match val & 0b1111111 {
            0b0110111 => Opcode::LUI,
            0b0010111 => Opcode::AUIPC,
            0b0010011 => Opcode::ADDI,
            0b1101111 => Opcode::JAL,
            0b0110011 => {
                // We know it is an R-Type
                let inst: Rtype = val.into();
                match inst.funct3 {
                    0b000 => Opcode::SUB,
                    _     => {
                        println!("{}: Unknown funct3", inst.funct3);
                        Opcode::None
                    }
                }
            }
            _ => Opcode::None,
        }
    }
}

/// An struct representing instruction
#[derive(Debug)]
pub struct Inst {
    pub typ: Type,
    pub opcode: Opcode,
}

impl From<u32> for Inst {
    fn from(val: u32) -> Self {
        let opcode: Opcode = val.into();
        let typ = match opcode {
            Opcode::LUI   => Type::U(val.into()),
            Opcode::AUIPC => Type::U(val.into()),
            Opcode::ADDI  => Type::I(val.into()),
            Opcode::SUB   => Type::R(val.into()),
            Opcode::JAL   => Type::J(val.into()),
            _ => Type::None,
        };
        Inst {
            typ,
            opcode,
        }
    }
}

/// Struct with the emulator state and register in it
pub struct EmuState {
    regs: [u32; 33],
}

impl EmuState {
    /// Struct initialization sequence
    pub fn new() -> Self {
        EmuState {
            regs: [0u32; 33],
        }
    }
}

/// An struct representing emulator which will get plugged with other 
/// components
pub struct Emu {
    pub mmu: Mmu,
    pub state: EmuState,
}

impl Emu {
    /// Struct initialization sequence
    pub fn new(size: usize) -> Self {
        Emu {
            mmu: Mmu::new(size),
            state: EmuState::new(),
        }
    }

    /// Get a register from the state
    pub fn reg(&self, register: Register) -> u32 {
        if register != Register::Zero {
            self.state.regs[register as usize]
        } else {
            0
        }
    }

}

impl fmt::Display for Emu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
r#"zero {:08x} ra {:08x} sp  {:08x} gp  {:08x}
tp   {:08x} t0 {:08x} t1  {:08x} t2  {:08x}
s0   {:08x} s1 {:08x} a0  {:08x} a1  {:08x}
a2   {:08x} a3 {:08x} a4  {:08x} a5  {:08x}
a6   {:08x} a7 {:08x} s2  {:08x} s3  {:08x}
s4   {:08x} s5 {:08x} s6  {:08x} s7  {:08x}
s8   {:08x} s9 {:08x} s10 {:08x} s11 {:08x}
t3   {:08x} t4 {:08x} t5  {:08x} t6  {:08x}
pc   {:08x}"#,
        self.reg(Register::Zero),
        self.reg(Register::Ra),
        self.reg(Register::Sp),
        self.reg(Register::Gp),
        self.reg(Register::Tp),
        self.reg(Register::T0),
        self.reg(Register::T1),
        self.reg(Register::T2),
        self.reg(Register::S0),
        self.reg(Register::S1),
        self.reg(Register::A0),
        self.reg(Register::A1),
        self.reg(Register::A2),
        self.reg(Register::A3),
        self.reg(Register::A4),
        self.reg(Register::A5),
        self.reg(Register::A6),
        self.reg(Register::A7),
        self.reg(Register::S2),
        self.reg(Register::S3),
        self.reg(Register::S4),
        self.reg(Register::S5),
        self.reg(Register::S6),
        self.reg(Register::S7),
        self.reg(Register::S8),
        self.reg(Register::S9),
        self.reg(Register::S10),
        self.reg(Register::S11),
        self.reg(Register::T3),
        self.reg(Register::T4),
        self.reg(Register::T5),
        self.reg(Register::T6),
        self.reg(Register::Pc))
    }
}
