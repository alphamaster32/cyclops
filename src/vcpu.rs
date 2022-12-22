use crate::emu::{Inst, Type, Opcode};

use core::fmt;
/// Struct representing the pipeline
/// all the functions operating on the pipeline will be implemented on
/// this struct
#[derive(Debug)]
pub struct PipeLine {
    pub fetch:     Stage,
    pub decode:    Stage,
    pub execute:   Stage,
    pub memory:    Stage,
    pub writeback: Stage,
}

impl PipeLine {
    /// Struct initialization sequence
    pub fn new() -> Self {
        PipeLine {
            fetch:     Stage::new(),
            decode:    Stage::new(),
            execute:   Stage::new(),
            memory:    Stage::new(),
            writeback: Stage::new(),
        }
    }
    /// Hazard detection sequence
    pub fn is_hazard(&self) -> bool {
        let fet = self.fetch.inst.as_ref();
        let exec = self.execute.inst.as_ref();
        let dec = self.decode.inst.as_ref();
        let mem = self.memory.inst.as_ref();
        let wb = self.writeback.inst.as_ref();
        if exec != None && dec != None {
            let exec = self.execute.inst.as_ref().unwrap();
            let dec = self.decode.inst.as_ref().unwrap();
            if (exec.rd() == dec.rs1() || exec.rd() == dec.rs2()) &&
                exec.rd().clone().is_some() {
                return true;
            }
        }
        if dec != None && mem != None {
            let dec = self.decode.inst.as_ref().unwrap();
            let mem = self.memory.inst.as_ref().unwrap();
            if (mem.rd() == dec.rs1() || mem.rd() == dec.rs2()) &&
                mem.rd().clone().is_some() {
                return true;
            }
        }
        //if dec != None && wb != None {
        //    let dec = self.decode.inst.as_ref().unwrap();
        //    let wb = self.writeback.inst.as_ref().unwrap();
        //    if wb.rd() == dec.rs1() {
        //        return true;
        //    }
        //}
        //if dec != None && fet != None {
        //    let dec = self.decode.inst.as_ref().unwrap();
        //    let fet = self.fetch.inst.as_ref().unwrap();
        //    if fet.rs1() == dec.rd() {
        //        return true;
        //    }
        //}
        false
    }
}

/// represents a stage in the pipeline
#[derive(Debug)]
pub struct Stage {
    /// Instruction in the stage that could be `None`
    pub inst: Option<Inst>,
}

//impl fmt::Display for Stage {
//    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//        if let Some(inst) = self.inst {
//            write!(f, "Stage -> Opcode: {}, rd: {}, rs1: {}, rs2: {}",
//                inst.opcode, inst.rd().unwrap(), inst.rs1().unwrap(),
//                inst.rs2().unwrap()
//            )
//        } else {
//            write!(f, "Stage -> Opcode: _, rd: _, rs1: _, rs2: _")
//        }
//    }
//}

impl Stage {
    /// Struct initialization sequence
    pub fn new() -> Self {
        Stage {
            inst:     None,
        }
    }
    /// Put a new instruction in stage
    pub fn add_inst(&mut self, inst: Option<Inst>) -> Option<Inst>{
        if self.inst == None {
            self.inst = inst;
            None
        } else {
            let prev = self.inst.clone();
            self.inst = inst;
            prev
        }
    }
}
