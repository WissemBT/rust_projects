use std::{io::{self, Write}, thread::panicking};

const MEMORY_SIZE: usize = 4096;
const NREGS: usize = 16;

const IP: usize = 0;

pub struct Machine {
    // Implement me!
    mach_mem : [u8;MEMORY_SIZE],
    regs: [u32;NREGS],

}

#[derive(Debug)]
pub enum MachineError {
    // Add some entries to represent errors!
    RegisterDoesntExist,
    ErrWritingToFd,
    NoEquivalentOpcode,
    NoEquivalentInstrAddress,
    StoreReachEndOfMemory,
    LoadReachEndOfMemory,

}

impl Machine {
    /// Create a new machine in its reset state. The `memory` parameter will
    /// be copied at the beginning of the machine memory.
    ///
    /// # Panics
    /// This function panics when `memory` is larger than the machine memory.
    pub fn new(memory: &[u8]) -> Self {
        // unimplemented!()  // Implement me!
        if memory.len() > MEMORY_SIZE { panic!("memory is larger than the machine memory");}
        let mut new_mach : Machine = Machine {mach_mem : [0;MEMORY_SIZE], regs: [0;NREGS]};
        new_mach.mach_mem[..memory.len()].copy_from_slice(&memory[..]);
        new_mach
    }

    /// Run until the program terminates or until an error happens.
    /// If output instructions are run, they print on `fd`.
    pub fn run_on<T: Write>(&mut self, fd: &mut T) -> Result<(), MachineError> {
        while !self.step_on(fd)? {}
        Ok(())
    }

    /// Run until the program terminates or until an error happens.
    /// If output instructions are run, they print on standard output.
    pub fn run(&mut self) -> Result<(), MachineError> {
        self.run_on(&mut io::stdout().lock())
    }

    /// Execute the next instruction by doing the following steps:
    ///   - decode the instruction located at IP (register 0)
    ///   - increment the IP by the size of the instruction
    ///   - execute the decoded instruction
    ///
    /// If output instructions are run, they print on `fd`.
    /// If an error happens at either of those steps, an error is
    /// returned.
    ///
    /// In case of success, `true` is returned if the program is
    /// terminated (upon encountering an exit instruction), or
    /// `false` if the execution must continue.
    pub fn step_on<T: Write>(&mut self, fd: &mut T) -> Result<bool, MachineError> {
        // unimplemented!()  // Implement me!
        let instr_addr: usize = self.regs[IP] as usize;
        if instr_addr >= MEMORY_SIZE {
            return Err(MachineError::NoEquivalentInstrAddress)
        }
        let decoded_instr = self.mach_mem[instr_addr];
        match decoded_instr {
            //move if
            1 => {
                let reg_a = self.mach_mem[instr_addr+1];
                let reg_b = self.mach_mem[instr_addr+2];
                let reg_c = self.mach_mem[instr_addr+3];
                self.regs[IP]+=4;
                if reg_a > 15 || reg_b > 15 || reg_c >15 {
                    return Err(MachineError::RegisterDoesntExist)
                }
                if self.regs[reg_c as usize]!=0 {
                    self.regs[reg_a as usize] = self.regs[reg_b as usize];
                }
                Ok(false)
            }
            //store
            2 => {
                let reg_a = self.mach_mem[instr_addr+1];
                let reg_b = self.mach_mem[instr_addr+2];
                self.regs[IP]+=3;
                if reg_a > 15 || reg_b >15 {
                    return Err(MachineError::RegisterDoesntExist)
                }
                let reg_b_cont = self.regs[reg_b as usize].to_le_bytes();
                let adr : usize = self.regs[reg_a as usize] as usize;
                if adr > MEMORY_SIZE-4{
                    return Err(MachineError::StoreReachEndOfMemory);
                } 
                self.mach_mem[adr] = reg_b_cont[0];
                self.mach_mem[adr+1] = reg_b_cont[1];
                self.mach_mem[adr+2] = reg_b_cont[2];
                self.mach_mem[adr+3] = reg_b_cont[3];
                Ok(false)
            }
            //load
            3 => {
                let reg_a = self.mach_mem[instr_addr+1];
                let reg_b = self.mach_mem[instr_addr+2];
                self.regs[IP]+=3;
                if reg_a > 15 || reg_b >15 {
                    return Err(MachineError::RegisterDoesntExist)
                }
                let adr : usize = self.regs[reg_b as usize] as usize;
                if adr > MEMORY_SIZE-4{
                    return Err(MachineError::LoadReachEndOfMemory);
                } 
                let mem_cont = [self.mach_mem[adr],self.mach_mem[adr+1],self.mach_mem[adr+2],self.mach_mem[adr+3]];
                let value = u32::from_le_bytes(mem_cont);
                self.regs[reg_a as usize] = value;
                Ok(false)
            }
            //loadimm
            4 => {
                let reg_a = self.mach_mem[instr_addr+1];
                let l = self.mach_mem[instr_addr+2];
                let h = self.mach_mem[instr_addr+3];
                self.regs[IP]+=4;
                if reg_a > 15 {
                    return Err(MachineError::RegisterDoesntExist)
                }
                let signed_val = i16::from_le_bytes([l,h]) as i32;
                self.regs[reg_a as usize] = signed_val as u32;
                Ok(false)
            }
            //sub
            5 => {
                let reg_a = self.mach_mem[instr_addr+1];
                let reg_b = self.mach_mem[instr_addr+2];
                let reg_c = self.mach_mem[instr_addr+3];
                self.regs[IP]+=4;
                if reg_a > 15 || reg_b > 15 || reg_c > 15 {
                    return Err(MachineError::RegisterDoesntExist)
                }
                self.regs[reg_a as usize] = self.regs[reg_b as usize].wrapping_sub(self.regs[reg_c as usize]);
                Ok(false)    
            }
            //out
            6 => {
                let reg_a = self.mach_mem[instr_addr+1];
                self.regs[IP]+=2;
                if reg_a > 15 {
                    return Err(MachineError::RegisterDoesntExist)
                }
                let reg_cont = self.regs[reg_a as usize].to_le_bytes();
                let chr = reg_cont[0] as char;
                match write!(fd,"{}",chr) {
                     Err(_) => return Err(MachineError::ErrWritingToFd),
                     Ok(_) => return Ok(false)
                };
            }
            //exit
            7 => {
                self.regs[IP]+=1;
                Ok(true)
            }
            //out number
            8 => {
                let reg_a = self.mach_mem[instr_addr+1];
                self.regs[IP]+=2;
                if reg_a > 15 {
                    return Err(MachineError::RegisterDoesntExist)
                }
                let reg_cont = self.regs[reg_a as usize] as i32;
                match write!(fd,"{}",reg_cont) {
                     Err(_) => return Err(MachineError::ErrWritingToFd),
                     Ok(_) => return Ok(false)
                };
            }
            _ => {
                self.regs[IP]+=1;
                Err(MachineError::NoEquivalentOpcode)
            }

            
            
           

        }
    }

    /// Similar to [step_on](Machine::step_on).
    /// If output instructions are run, they print on standard output.
    pub fn step(&mut self) -> Result<bool, MachineError> {
        self.step_on(&mut io::stdout().lock())
    }

    /// Reference onto the machine current set of registers.
    pub fn regs(&self) -> &[u32] {
        // unimplemented!()  // Implement me!
        &self.regs
    }

    /// Sets a register to the given value.
    pub fn set_reg(&mut self, reg: usize, value: u32) -> Result<(), MachineError> {
        // unimplemented!()  // Implement me!
        if reg > 15 {
            return Err(MachineError::RegisterDoesntExist);
        }
        self.regs[reg] = value;
        Ok(())
    }

    /// Reference onto the machine current memory.
    pub fn memory(&self) -> &[u8] {
        // unimplemented!()  // Implement me!
        &self.mach_mem
    }
}
