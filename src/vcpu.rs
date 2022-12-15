use crate::emu::{Inst, Type};
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
        let exmem = self.execute.inst.as_ref();
        let idex = self.decode.inst.as_ref();
        let memwb = self.memory.inst.as_ref();
        if exmem != None && idex != None {
            let exmem = self.execute.inst.as_ref().unwrap();
            let idex = self.decode.inst.as_ref().unwrap();
            if exmem.rd() == idex.rs1() {
                println!("HAZARD");
                return true;
            }
        }
        if idex != None && memwb != None {
            let idex = self.decode.inst.as_ref().unwrap();
            let memwb = self.memory.inst.as_ref().unwrap();
            if memwb.rd() == idex.rs1() {
                println!("HAZARD");
                return true;
            }
        }
        false
    }
}

/// represents a stage in the pipeline
#[derive(Debug)]
pub struct Stage {
    /// Instruction in the stage that could be `None`
    pub inst: Option<Inst>,
}

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
            let prev = self.inst.take();
            self.inst = inst;
            prev
        }
    }
}
