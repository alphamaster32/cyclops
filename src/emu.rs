use crate::mmu::{Mmu, VirtAddr};
use crate::vcpu::PipeLine;
use crate::Error;

use core::fmt;
use core::sync::atomic::{AtomicUsize, Ordering};

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

/// File list in the emulator
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum File {
    Stdin,
    Stdout,
    Stderr,
}

/// This struct represents a transfer of state on the `EmuState`
#[derive(Debug, Clone)]
pub struct RegisterTransfer {
    reg:     Register,
    val:     u32,
    //pc:  u32
    is_exit: bool
}

/// This struct represents a transfer of state on the `Mmu`
#[derive(Debug, Clone)]
pub struct MemoryTransfer {
    loc:  usize,
    val:  [u8; 4],
    pc:   u32,
    size: usize,
    read: bool
}

/// An R-type instruction
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
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
    JAL,
    JALR,
    BEQ,
    BNE,
    BLT,
    BGE,
    BLTU,
    BGEU,
    LB,
    LH,
    LW,
    LBU,
    LHU,
    SB,
    SH,
    SW,
    ADDI,
    SLTI,
    SLTIU,
    XORI,
    ORI,
    ANDI,
    SLLI,
    SRLI,
    SRAI,
    ADD,
    SUB,
    SLL,
    SLT,
    SLTU,
    XOR,
    SRL,
    SRA,
    OR,
    AND,
    // M extention
    MUL,
    MULH,
    MULHSU,
    MULHU,
    DIV,
    DIVU,
    REM,
    REMU,
    // Not implementing fences
    ECALL,
    EBREAK
}

impl From<u32> for Opcode {
    fn from(val: u32) -> Self {
        match val & 0b1111111 {
            0b0110111 => Opcode::LUI,
            0b0010111 => Opcode::AUIPC,
            0b0010011 => {
                // We know it's a I-Type
                let inst: Itype = val.into();
                match inst.funct3 {
                    0b000 => Opcode::ADDI,
                    0b010 => Opcode::SLTI,
                    0b011 => Opcode::SLTIU,
                    0b100 => Opcode::XORI,
                    0b110 => Opcode::ORI,
                    0b111 => Opcode::ANDI,
                    0b001 => Opcode::SLLI,
                    0b101 => {
                        // A very odd choice that these opcodes are R-Type
                        match inst.imm >> 5 {
                            0b0000000 => Opcode::SRLI,
                            0b0100000 => Opcode::SRAI,
                            _         => Opcode::None
                        }
                    },
                    _     => Opcode::None
                }
            },
            0b1101111 => Opcode::JAL,
            0b1100111 => Opcode::JALR,
            0b1100011 => {
                // We know it's a B-Type
                let inst: Btype = val.into();
                match inst.funct3 {
                    0b000 => Opcode::BEQ,
                    0b001 => Opcode::BNE,
                    0b100 => Opcode::BLT,
                    0b101 => Opcode::BGE,
                    0b110 => Opcode::BLTU,
                    0b111 => Opcode::BGEU,
                    _     => Opcode::None,
                }
            },
            0b0000011 => {
                // We know it's a I-Type
                let inst: Itype = val.into();
                match inst.funct3 {
                    0b000 => Opcode::LB,
                    0b001 => Opcode::LH,
                    0b010 => Opcode::LW,
                    0b100 => Opcode::LBU,
                    0b101 => Opcode::LHU,
                    _     => Opcode::None
                }
            },
            0b0100011 => {
                // We know it's a S-Type
                let inst: Stype = val.into();
                match inst.funct3 {
                    0b000 => Opcode::SB,
                    0b001 => Opcode::SH,
                    0b010 => Opcode::SW,
                    _     => Opcode::None,
                }
            },
            0b0110011 => {
                // We know it's a R-Type
                let inst: Rtype = val.into();
                match inst.funct3 {
                    0b000 => {
                        match inst.funct7 {
                            0b0000000 => Opcode::ADD,
                            0b0100000 => Opcode::SUB,
                            0b0000001 => Opcode::MUL,
                            _         => Opcode::None,
                        }
                    },
                    0b001 => {
                        match inst.funct7 {
                            0b0000001 => Opcode::MULH,
                            0b0000000 => Opcode::SLL,
                            _         => Opcode::None,
                        }
                    },
                    0b010 => {
                        match inst.funct7 {
                            0b0000000 => Opcode::SLT,
                            0b0000001 => Opcode::MULHSU,
                            _         => Opcode::None,
                        }
                    },
                    0b011 => {
                        match inst.funct7 {
                            0b0000000 => Opcode::SLTU,
                            0b0000001 => Opcode::MULHU,
                            _         => Opcode::None,
                        }
                    },
                    0b100 => {
                        match inst.funct7 {
                            0b0000000 => Opcode::XOR,
                            0b0000001 => Opcode::DIV,
                            _         => Opcode::None,
                        }
                    },
                    0b101 => {
                        match inst.funct7 {
                            0b0000000 => Opcode::SRL,
                            0b1000000 => Opcode::SRA,
                            0b0000001 => Opcode::DIVU,
                            _         => Opcode::None,
                        }
                    },
                    0b110 => {
                        match inst.funct7 {
                            0b0000000 => Opcode::OR,
                            0b0000001 => Opcode::REM,
                            _         => Opcode::None,
                        }
                    }
                    0b111 => {
                        match inst.funct7 {
                            0b0000000 => Opcode::AND,
                            0b0000001 => Opcode::REMU,
                            _         => Opcode::None,
                        }
                    },
                    _     => Opcode::None,
                }
            },
            0b0001111 => {
                unreachable!()
            },
            0b1110011 => {
                // ECALL and EBREAK
                let inst: Itype = val.into();
                match inst.imm {
                    0b000000000000 => Opcode::ECALL,
                    0b000000000001 => Opcode::EBREAK,
                    _              => Opcode::None
                }
            }
            _ => Opcode::None,
        }
    }
}

/// An struct representing instruction
#[derive(Debug, Clone, PartialEq)]
pub struct Inst {
    pub typ:    Type,
    pub opcode: Opcode,
    pub pc:     u32,
    pub state:  Option<EmuState>
}

impl From<u32> for Inst {
    fn from(val: u32) -> Self {
        let opcode: Opcode = val.into();
        let typ = match opcode {
            Opcode::LUI       => Type::U(val.into()),
            Opcode::AUIPC     => Type::U(val.into()),
            Opcode::JAL       => Type::J(val.into()),
            Opcode::JALR      => Type::I(val.into()),
            Opcode::BEQ       => Type::B(val.into()),
            Opcode::BNE       => Type::B(val.into()),
            Opcode::BLT       => Type::B(val.into()),
            Opcode::BGE       => Type::B(val.into()),
            Opcode::BLTU      => Type::B(val.into()),
            Opcode::BGEU      => Type::B(val.into()),
            Opcode::LB        => Type::I(val.into()),
            Opcode::LH        => Type::I(val.into()),
            Opcode::LW        => Type::I(val.into()),
            Opcode::LBU       => Type::I(val.into()),
            Opcode::LHU       => Type::I(val.into()),
            Opcode::SB        => Type::S(val.into()),
            Opcode::SH        => Type::S(val.into()),
            Opcode::SW        => Type::S(val.into()),
            Opcode::ADDI      => Type::I(val.into()),
            Opcode::SLTI      => Type::I(val.into()),
            Opcode::SLTIU     => Type::I(val.into()),
            Opcode::XORI      => Type::I(val.into()),
            Opcode::ORI       => Type::I(val.into()),
            Opcode::ANDI      => Type::I(val.into()),
            Opcode::SLLI      => Type::I(val.into()),
            Opcode::SRLI      => Type::I(val.into()),
            Opcode::SRAI      => Type::I(val.into()),
            Opcode::ADD       => Type::R(val.into()),
            Opcode::SUB       => Type::R(val.into()),
            Opcode::SLL       => Type::R(val.into()),
            Opcode::SLT       => Type::R(val.into()),
            Opcode::SLTU      => Type::R(val.into()),
            Opcode::XOR       => Type::R(val.into()),
            Opcode::SRL       => Type::R(val.into()),
            Opcode::SRA       => Type::R(val.into()),
            Opcode::OR        => Type::R(val.into()),
            Opcode::AND       => Type::R(val.into()),
            Opcode::ECALL     => Type::I(val.into()),
            Opcode::EBREAK    => Type::I(val.into()),
            Opcode::MUL       => Type::R(val.into()),
            Opcode::MULH      => Type::R(val.into()),
            Opcode::MULHSU    => Type::R(val.into()),
            Opcode::MULHU     => Type::R(val.into()),
            Opcode::DIV       => Type::R(val.into()),
            Opcode::DIVU      => Type::R(val.into()),
            Opcode::REM       => Type::R(val.into()),
            Opcode::REMU      => Type::R(val.into()),
            _                 => Type::None,
        };
        Inst {
            typ,
            opcode,
            state: None,
            pc: 0
        }
    }
}

impl Inst {
    /// Get the destination register
    pub fn rd(&self) -> Option<Register> {
        match &self.typ {
            Type::U(inst) => Some(inst.rd),
            Type::I(inst) => Some(inst.rd),
            Type::B(inst) => None,
            Type::J(inst) => Some(inst.rd),
            Type::S(inst) => None,
            Type::R(inst) => Some(inst.rd),
            _             => None
        }
    }
    /// Get the first source register
    pub fn rs1(&self) -> Option<Register> {
        match &self.typ {
            Type::U(inst) => None,
            Type::I(inst) => Some(inst.rs1),
            Type::B(inst) => Some(inst.rs1),
            Type::J(inst) => None,
            Type::S(inst) => Some(inst.rs1),
            Type::R(inst) => Some(inst.rs1),
            _             => None
        }
    }
    /// Get the second source register
    pub fn rs2(&self) -> Option<Register> {
        match &self.typ {
            Type::U(inst) => None,
            Type::I(inst) => None,
            Type::B(inst) => Some(inst.rs2),
            Type::J(inst) => None,
            Type::S(inst) => Some(inst.rs2),
            Type::R(inst) => Some(inst.rs2),
            _             => None
        }
    }
    /// Get the immediate
    pub fn imm(&self) -> Option<i32> {
        match &self.typ {
            Type::U(inst) => Some(inst.imm),
            Type::I(inst) => Some(inst.imm),
            Type::B(inst) => Some(inst.imm),
            Type::J(inst) => Some(inst.imm),
            Type::S(inst) => Some(inst.imm),
            Type::R(inst) => None,
            _             => None
        }
    }
}

/// Struct with the emulator state and register in it
#[derive(Debug, Clone, Copy, PartialEq)]
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
    /// Get a register from the state
    pub fn reg(&self, register: Register) -> u32 {
        if register != Register::Zero {
            self.regs[register as usize]
        } else {
            0
        }
    }

    pub fn set(&mut self, register: Register, val: u32) {
        self.regs[register as usize] = val;
    }

    pub fn commit(&mut self, rt: RegisterTransfer) {
        self.set(rt.reg, rt.val);
    }
}

/// An struct representing emulator which will get plugged with other 
/// components
pub struct Emu {
    pub mmu:             Mmu,
    pub state:           EmuState,
    pub pipe:            PipeLine,
    pub pc:              usize,
    pub cycles:          AtomicUsize,
    pub files:           Vec<Option<File>>,
    pub uncommited_mems: Vec<MemoryTransfer>,
    pub uncommited_regs: Vec<RegisterTransfer>
}

impl Emu {
    /// Struct initialization sequence
    pub fn new(size: usize) -> Self {
        Emu {
            mmu:             Mmu::new(size),
            state:           EmuState::new(),
            pipe:            PipeLine::new(),
            pc:              0,
            cycles:          AtomicUsize::new(1),
            files:           vec![
                Some(File::Stdin),
                Some(File::Stdout),
                Some(File::Stderr)
            ],
            uncommited_mems: vec![],
            uncommited_regs: vec![]
        }
    }
    pub fn get_fd(&mut self, fd: usize) -> Option<&mut File> {
        self.files.get_mut(fd).unwrap().as_mut()
    }
    pub fn cycle(&mut self) -> Result<(), Error> {
        //Fetch the instruction
        let pc = self.state.reg(Register::Pc);
        //println!("{:?}", pc.checked_add(4));
        let mut inst: Inst = u32::from_le_bytes(self.mmu.memory.get(
                (pc as usize)..pc.checked_add(4)
                .ok_or(Error::ArithmeticException)? as usize)
                .ok_or(Error::ArithmeticException)?
                .try_into().map_err(|_err| Error::ArithmeticException)?)
                .into();
        inst.state= None;
        inst.pc = pc;

        // Fetch the first instrction
        let mut next: Option<Inst> = None;
        // If there is not a hazard proceed
        if !self.pipe.is_hazard() {
            next = self.pipe.fetch.add_inst(Some(inst));
            let pc = self.state.reg(Register::Pc);
            self.state.set(Register::Pc, pc + 4);
        }

        // Move the instructions in the pipeline
        // We read the registers in this stage
        if next.clone().is_some() && !self.pipe.is_hazard() {
            let mut inst = next.unwrap();
            inst.state = Some(self.state.clone());
            //println!("{:?}", inst);
            next = self.pipe.decode.add_inst(Some(inst));
        } else {
            next = None;
        }
        // Before moving the instruction we should execute it
        if next.clone().is_some() {
            let (rt, mt) = self.run_inst(next.clone());
            if rt.clone().is_some() {
                self.uncommited_regs.push(rt.unwrap());
            }
            if mt.clone().is_some() {
                self.uncommited_mems.push(mt.unwrap());
            }
            //println!("{:#}", self);
            //println!("inst: {:?}, pc: {:x?}", next.clone().unwrap().opcode,
            //next.clone().unwrap().pc);
        }
        //self.uncommited_mems.push(mt);
        let next = self.pipe.execute.add_inst(next.clone());
        // Access the memory in this stage
        //self.state.commit(self.)
        let next = self.pipe.memory.add_inst(next);
        if self.uncommited_mems.len() > 0 {
            let mt = self.uncommited_mems.remove(0);
            self.mmu.write(VirtAddr(mt.loc), mt.size, mt.val);
        }
        // We write the registers in this stage
        let next = self.pipe.writeback.add_inst(next);
        if self.uncommited_regs.len() > 0 {
            let state = self.uncommited_regs.remove(0);
            if state.is_exit {
                return Err(Error::EmuExit)
            }
            self.state.commit(state);
            // Simulate the half cycle in the writeback stage
            if self.pipe.decode.inst != None {
                let mut inst = self.pipe.decode.inst.clone().unwrap();
                inst.state = Some(self.state.clone());
                self.pipe.decode.inst = Some(inst);
            }
        }
        // Increment atomicaly for future multi-thread update
        self.cycles.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
    /// Run the selected instruction on the emulator
    /// Where this instruction is executed is dependent on the pipeline stage
    pub fn run_inst(&mut self, inst: Option<Inst>) -> 
        (Option<RegisterTransfer>, Option<MemoryTransfer>) {
        // Copy the state
        //let mmu = self.mmu.clone();
        //let emustate = self.state.clone();
        let mut rt = RegisterTransfer {
            reg: Register::Zero,
            val: 0,
            is_exit: false
            //pc: emustate.reg(Register::Pc)
        };
        let mut mt = MemoryTransfer {
            loc:  0,
            val:  [0, 0, 0, 0],
            pc:   0,
            size: 0,
            read: false
        };

        // Check for the opcode
        // Possible panic place
        let inst = inst.unwrap();
        let is_exit: bool = false;
        match inst.opcode {
            Opcode::AUIPC => {
                //FIXME Possible source of bug in the pipeline
                rt.val = (inst.pc as i32 )
                    .wrapping_add(inst.imm().unwrap() as i32) as u32;
                rt.reg = inst.rd().unwrap();
                return (Some(rt), None);
            },
            Opcode::ADDI => {
                rt.val = (inst.state.unwrap().reg(inst.rs1().unwrap()) as i32)
                    .wrapping_add(inst.imm().unwrap() as i32) as u32;
                rt.reg = inst.rd().unwrap();
                return (Some(rt), None);
            },
            Opcode::SUB => {
                rt.val = (inst.state.unwrap().reg(inst.rs1().unwrap()) as i32 -
                inst.state.unwrap().reg(inst.rs2().unwrap()) as i32) as u32;
                rt.reg = inst.rd().unwrap();
                return (Some(rt), None);
            },
            Opcode::JAL => {
                rt.val = (inst.pc + 0x4) as u32;
                rt.reg = inst.rd().unwrap();
                self.state.set(Register::Pc, inst.pc.wrapping_add(
                        inst.imm().unwrap() as u32
                        ));
                self.pipe.decode.inst = None;
                self.pipe.fetch.inst = None;
                //println!("{:x?}",inst);
                return (Some(rt), None);
            },
            Opcode::BGEU => {
                let rs1 = inst.state.unwrap().reg(inst.rs1().unwrap());
                let rs2 = inst.state.unwrap().reg(inst.rs2().unwrap());
                if (rs1 as u32) >= (rs2 as u32) {
                    self.state.set(Register::Pc, inst.pc + inst.imm().unwrap() 
                        as u32);
                    self.pipe.decode.inst = None;
                    self.pipe.fetch.inst = None;
                } else {
                    rt.val = 0;
                    rt.reg = Register::Zero;
                }
                return (Some(rt), None);
            },
            Opcode::ANDI => {
                rt.val = (inst.state.unwrap().reg(inst.rs1().unwrap()) as u32)
                    & (inst.imm().unwrap() as u32);
                rt.reg = inst.rd().unwrap();
                return (Some(rt), None);
            },
            Opcode::BNE => {
                let rs1 = inst.state.unwrap().reg(inst.rs1().unwrap());
                let rs2 = inst.state.unwrap().reg(inst.rs2().unwrap());
                if (rs1 as i32) != (rs2 as i32) {
                    self.state.set(Register::Pc, inst.pc + 
                        inst.imm().unwrap() as u32);
                    self.pipe.decode.inst = None;
                    self.pipe.fetch.inst = None;
                } else {
                    rt.val = 0;
                    rt.reg = Register::Zero;
                }
                return (Some(rt), None);
            },
            Opcode::SLLI => {
                let shamt = inst.imm().unwrap() as u32 & 0b111111;
                rt.val = (inst.state.unwrap().reg(inst.rs1().unwrap()) as u32) 
                    << shamt;
                rt.reg = inst.rd().unwrap();
                return (Some(rt), None);
            },
            Opcode::ADD => {
                rt.val = ((inst.state.unwrap().reg(inst.rs1().unwrap()) as i32)
                    .wrapping_add(inst.state.unwrap()
                        .reg(inst.rs2().unwrap()) as i32)) as u32;
                rt.reg = inst.rd().unwrap();
                return (Some(rt), None);
            },
            Opcode::JALR => {
                let target = (inst.state.unwrap().reg(inst.rs1().unwrap())
                    as u32).wrapping_add(inst.imm().unwrap() as u32);
                rt.val = inst.pc + 4;
                rt.reg = inst.rd().unwrap();
                self.state.set(Register::Pc, target);
                self.pipe.decode.inst = None;
                self.pipe.fetch.inst = None;
                return (Some(rt), None);
            },
            Opcode::SB => {
                let loc = inst.state.unwrap().reg(inst.rs1().unwrap())
                    .wrapping_add(inst.imm().unwrap() as u32);
                let val = inst.state.unwrap().reg(inst.rs2().unwrap())
                    .to_le_bytes();
                mt.loc = loc as usize;
                mt.val = val;
                mt.size = 1;
                return (None, Some(mt));
            },
            Opcode::SW => {
                let loc = inst.state.unwrap().reg(inst.rs1().unwrap())
                    .wrapping_add(inst.imm().unwrap() as i32 as u32);
                let val = inst.state.unwrap().reg(inst.rs2().unwrap())
                    .to_le_bytes();
                mt.loc = loc as usize;
                mt.val = val;
                mt.size = 4;
                return (None, Some(mt));
            },
            Opcode::BLTU => {
                let rs1 = inst.state.unwrap().reg(inst.rs1().unwrap());
                let rs2 = inst.state.unwrap().reg(inst.rs2().unwrap());
                if (rs1 as u32) < (rs2 as u32){
                    self.state.set(Register::Pc, (inst.pc)
                        .wrapping_add(inst.imm().unwrap() as u32));
                    self.pipe.decode.inst = None;
                    self.pipe.fetch.inst = None;
                } else {
                    rt.val = 0;
                    rt.reg = Register::Zero;
                }
                return (Some(rt), None);
            },
            Opcode::BEQ => {
                let rs1 = inst.state.unwrap().reg(inst.rs1().unwrap());
                let rs2 = inst.state.unwrap().reg(inst.rs2().unwrap());
                if (rs1 as i32) == (rs2 as i32) {
                    self.state.set(Register::Pc, (inst.pc)
                        .wrapping_add(inst.imm().unwrap() as u32));
                    self.pipe.decode.inst = None;
                    self.pipe.fetch.inst = None;
                } else {
                    rt.val = 0;
                    rt.reg = Register::Zero;
                }
                return (Some(rt), None);
            },
            Opcode::LW => {
                let addr = inst.state.unwrap().reg(inst.rs1().unwrap())
                    .wrapping_add(inst.imm().unwrap() as u32) as usize;
                let mut tmp = [0u8; 4];
                rt.val = i32::from_le_bytes(self.mmu
                    .read_word(VirtAddr(addr))) as u32;
                rt.reg = inst.rd().unwrap();
                return (Some(rt), None);
            },
            Opcode::BLT => {
                let rs1 = inst.state.unwrap().reg(inst.rs1().unwrap());
                let rs2 = inst.state.unwrap().reg(inst.rs2().unwrap());
                if (rs1 as i32) < (rs2 as i32){
                    self.state.set(Register::Pc, inst.pc + inst.imm().unwrap() 
                        as u32);
                    self.pipe.decode.inst = None;
                    self.pipe.fetch.inst = None;
                } else {
                    rt.val = 0;
                    rt.reg = Register::Zero;
                }
                return (Some(rt), None);
            },
            Opcode::LUI => {
                rt.val = inst.imm().unwrap() as u32;
                rt.reg = inst.rd().unwrap();
                return (Some(rt), None);
            },
            Opcode::SRAI => {
                let shamt = inst.imm().unwrap() & 0b111111;
                rt.val = ((inst.state.unwrap().reg(inst.rs1().unwrap()) as i32) 
                    >> shamt) as u32;
                rt.reg = inst.rd().unwrap();
                return (Some(rt), None);
            },
            Opcode::AND => {
                rt.val = (inst.state.unwrap().reg(inst.rs1().unwrap()) as u32 &
                inst.state.unwrap().reg(inst.rs2().unwrap()) as u32);
                rt.reg = inst.rd().unwrap();
                return (Some(rt), None);
            },
            Opcode::OR => {
                rt.val = (inst.state.unwrap().reg(inst.rs1().unwrap()) as u32 |
                inst.state.unwrap().reg(inst.rs2().unwrap()) as u32);
                rt.reg = inst.rd().unwrap();
                return (Some(rt), None);
            },
            Opcode::LBU => {
                let addr = inst.state.unwrap().reg(inst.rs1().unwrap())
                    .wrapping_add(inst.imm().unwrap() as u32) as usize;
                let mut tmp = [0u8; 1];
                rt.val = u8::from_le_bytes(self.mmu
                    .read_byte(VirtAddr(addr))) as u32;
                rt.reg = inst.rd().unwrap();
                return (Some(rt), None);
            },
            Opcode::LH => {
                let addr = inst.state.unwrap().reg(inst.rs1().unwrap())
                    .wrapping_add(inst.imm().unwrap() as u32) as usize;
                let mut tmp = [0u8; 2];
                rt.val = i16::from_le_bytes(self.mmu
                    .read_half(VirtAddr(addr))) as u32;
                rt.reg = inst.rd().unwrap();
                return (Some(rt), None);
            },
            Opcode::SH => {
                let loc = inst.state.unwrap().reg(inst.rs1().unwrap())
                    .wrapping_add(inst.imm().unwrap() as u32);
                let val = inst.state.unwrap().reg(inst.rs2().unwrap())
                    .to_le_bytes();
                mt.loc = loc as usize;
                mt.val = val;
                mt.size = 2;
                return (None, Some(mt));
            },
            Opcode::ECALL => {
                // We've got a syscall
                // Get the syscall number from the A7 register
                let num = inst.state.unwrap().reg(Register::A7);

                match num {
                    // fstat()
                    80  => {
                        // Get the arguments
                        let fd = inst.state.unwrap().reg(Register::A0);
                        let statbuf = inst.state.unwrap().reg(Register::A1);

                        // Define the kernel Stat64
                        #[repr(C)]
                        #[derive(Default, Debug)]
                        struct Stat {
                            st_dev:     u64,
                            st_ino:     u64,
                            st_mode:    u32,
                            st_nlink:   u32,
                            st_uid:     u32,
                            st_gid:     u32,
                            st_rdev:    u64,
                            __pad1:     u64,

                            st_size:    i64,
                            st_blksize: i32,
                            __pad2:     i32,

                            st_blocks: i64,

                            st_atime:     u64,
                            st_atimensec: u64,
                            st_mtime:     u64,
                            st_mtimensec: u64,
                            st_ctime:     u64,
                            st_ctimensec: u64,

                            __glibc_reserved: [i32; 2],
                        }

                        // See if the file exists
                        let file = self.get_fd(fd as usize);
                        if file.is_none() || file.as_ref().is_none() {
                            // FD was not valid, return out with an error
                            rt.val = !0;
                            rt.reg = Register::A0;
                            return (Some(rt), None);
                        }

                        // If it is a file do stuff for now we do not
                        // have a file

                        // For stdXX we do not need to return anything
                        // This changes the execution path
                        rt.val = 0;
                        rt.reg = Register::A0;
                        return (Some(rt), None);
                    },
                    // brk
                    214 => {
                        let req = inst.state.unwrap().reg(Register::A0);
                        let cur_base = self.mmu.alloc_base;

                        let inc = if req != 0 {
                            (req as i32).checked_sub(cur_base.0 as i32)
                                .unwrap()
                        } else {
                            0
                        };

                        // No negative brk
                        assert!(inc >= 0);
                        if let Some(_) = self.mmu.allocate(inc as usize) {
                            let new_base = cur_base.0 + inc as usize;
                            rt.val = new_base as u32;
                            rt.reg = Register::A0;
                        } else {
                            rt.val = !0;
                            rt.reg = Register::A0;
                        }
                        return (Some(rt), None);
                    },
                    // write
                    64  => {
                        let fd = inst.state.unwrap().reg(Register::A0);
                        let buf = inst.state.unwrap().reg(Register::A1);
                        let len = inst.state.unwrap().reg(Register::A2);

                        let file = self.get_fd(fd as usize);
                        if let Some(file) = file {
                            if file == &File::Stdout || file == &File::Stderr {
                                let bytes =
                                    self.mmu.read(VirtAddr(buf as usize),
                                    len as usize).unwrap();
                                print!("{}", core::str::from_utf8(bytes).unwrap());
                            }
                        } else {
                            rt.val = !0;
                            rt.reg = Register::A0;
                        }
                        rt.val = 0;
                        rt.reg = Register::A0;
                        return (Some(rt), None);
                    },
                    // close
                    57  => {
                        let fd = inst.state.unwrap().reg(Register::A0);
                        let file = self.get_fd(fd as usize);
                        if file.is_some() {
                            //*file = None;
                            rt.val = 0;
                            rt.reg = Register::A0;
                        } else {
                            rt.val = !0;
                            rt.reg = Register::A0;
                        }
                        return(Some(rt), None);
                    },
                    // exit
                    93  => {
                        rt.is_exit = true;
                        return(Some(rt), None);
                    }
                    _  => {
                        unimplemented!("unspecified syscall name {:?}", num);
                    }
                }

            },
            Opcode::LHU => {
                let addr = inst.state.unwrap().reg(inst.rs1().unwrap())
                    .wrapping_add(inst.imm().unwrap() as u32) as usize;
                let mut tmp = [0u8; 2];
                rt.val = u16::from_le_bytes(self.mmu
                    .read_half(VirtAddr(addr))) as u32;
                rt.reg = inst.rd().unwrap();
                return (Some(rt), None);
            },
            Opcode::SRLI => {
                let shamt = inst.imm().unwrap() & 0b111111;
                rt.val = (inst.state.unwrap().reg(inst.rs1().unwrap()) as u32) 
                    >> shamt;
                rt.reg = inst.rd().unwrap();
                return (Some(rt), None);
            },
            Opcode::SLL => {
                let shamt = inst.state.unwrap()
                    .reg(inst.rs2().unwrap()) as u32 & 0b111111;
                rt.val = (inst.state.unwrap().reg(inst.rs1().unwrap()) as u32) 
                    << shamt;
                rt.reg = inst.rd().unwrap();
                return (Some(rt), None);
            },
            Opcode::SLTIU => {
                if (inst.rs1().unwrap() as u32) <
                    (inst.imm().unwrap() as u32) {
                        rt.val = 1;
                    } else {
                        rt.val = 0;
                    }
                rt.reg = inst.rd().unwrap();
                return (Some(rt), None);
            },
            Opcode::ORI => {
                rt.val = inst.state.unwrap().reg(inst.rs1().unwrap()) |
                    inst.imm().unwrap() as u32;
                rt.reg = inst.rd().unwrap();
                return (Some(rt), None);
            },
            Opcode::BGE => {
                let rs1 = inst.state.unwrap().reg(inst.rs1().unwrap());
                let rs2 = inst.state.unwrap().reg(inst.rs2().unwrap());
                if (rs1 as i32) >= (rs2 as i32){
                    self.state.set(Register::Pc, (inst.pc)
                        .wrapping_add(inst.imm().unwrap() as u32));
                    self.pipe.decode.inst = None;
                    self.pipe.fetch.inst = None;
                } else {
                    rt.val = 0;
                    rt.reg = Register::Zero;
                }
                return (Some(rt), None);
            },
            Opcode::XOR => {
                rt.val = (inst.state.unwrap().reg(inst.rs1().unwrap()) as u32 ^
                inst.state.unwrap().reg(inst.rs2().unwrap()) as u32);
                rt.reg = inst.rd().unwrap();
                return (Some(rt), None);
            },
            Opcode::XORI => {
                rt.val = inst.state.unwrap().reg(inst.rs1().unwrap()) ^
                    inst.imm().unwrap() as u32;
                rt.reg = inst.rd().unwrap();
                return (Some(rt), None);
            },
            Opcode::SLTU => {
                if (inst.state.unwrap().reg(inst.rs1().unwrap()) as u32) <
                    (inst.state.unwrap().reg(inst.rs2().unwrap()) as u32) {
                        rt.val = 1;
                    } else {
                        rt.val = 0;
                    }
                rt.reg = inst.rd().unwrap();
                return (Some(rt), None);
            },
            Opcode::MUL => {
                let val = (inst.state.unwrap().reg(inst.rs1().unwrap()) as i64 *
                inst.state.unwrap().reg(inst.rs2().unwrap()) as i64) as u64;
                rt.reg = inst.rd().unwrap();
                rt.val = ((val << 32) >> 32) as u32;
                return (Some(rt), None);
            },
            Opcode::REMU => {
                rt.val = (inst.state.unwrap().reg(inst.rs1().unwrap()) as u32 %
                inst.state.unwrap().reg(inst.rs2().unwrap()) as u32) as u32;
                rt.reg = inst.rd().unwrap();
                return (Some(rt), None);
            },
            Opcode::MULHU => {
                let val = (inst.state.unwrap().reg(inst.rs1().unwrap()) as u64 *
                inst.state.unwrap().reg(inst.rs2().unwrap()) as u64);
                rt.val = (val >> 32) as u32;
                rt.reg = inst.rd().unwrap();
                return (Some(rt), None);
            },
            Opcode::SRL => {
                let shamt = inst.state.unwrap()
                    .reg(inst.rs2().unwrap()) as u32 & 0b111111;
                rt.val = (inst.state.unwrap().reg(inst.rs1().unwrap()) as u32) 
                    >> shamt;
                rt.reg = inst.rd().unwrap();
                return (Some(rt), None);
            },
            Opcode::SLTI => {
                if (inst.state.unwrap().reg(inst.rs1().unwrap()) as i32) <
                    (inst.imm().unwrap() as i32) {
                        rt.val = 1;
                    } else {
                        rt.val = 0;
                    }
                rt.reg = inst.rd().unwrap();
                return (Some(rt), None);
            },
            Opcode::DIVU => {
                // FIXME handle division by zero
                let val1 = inst.state.unwrap().reg(inst.rs1().unwrap());
                let val2 = inst.state.unwrap().reg(inst.rs2().unwrap());
                //println!("rs1: {:?}, rs2: {:?}, val: {:?}, pc: {:x?}",
                    //inst.state.unwrap().reg(inst.rs1().unwrap()),
                    //inst.state.unwrap().reg(inst.rs2().unwrap()), val1 / val2,
                    //inst.pc);
                rt.val = val1 / val2;
                rt.reg = inst.rd().unwrap();
                //println!("{:x?}", inst.pc);
                return (Some(rt), None);
            },
            _             => {
                println!("{:#}", self);
                unimplemented!("unimplemented Opcode {:?}", inst.opcode)
            }
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
        self.state.reg(Register::Zero),
        self.state.reg(Register::Ra),
        self.state.reg(Register::Sp),
        self.state.reg(Register::Gp),
        self.state.reg(Register::Tp),
        self.state.reg(Register::T0),
        self.state.reg(Register::T1),
        self.state.reg(Register::T2),
        self.state.reg(Register::S0),
        self.state.reg(Register::S1),
        self.state.reg(Register::A0),
        self.state.reg(Register::A1),
        self.state.reg(Register::A2),
        self.state.reg(Register::A3),
        self.state.reg(Register::A4),
        self.state.reg(Register::A5),
        self.state.reg(Register::A6),
        self.state.reg(Register::A7),
        self.state.reg(Register::S2),
        self.state.reg(Register::S3),
        self.state.reg(Register::S4),
        self.state.reg(Register::S5),
        self.state.reg(Register::S6),
        self.state.reg(Register::S7),
        self.state.reg(Register::S8),
        self.state.reg(Register::S9),
        self.state.reg(Register::S10),
        self.state.reg(Register::S11),
        self.state.reg(Register::T3),
        self.state.reg(Register::T4),
        self.state.reg(Register::T5),
        self.state.reg(Register::T6),
        self.state.reg(Register::Pc))
    }
}
