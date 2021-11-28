use std::path;

use elf;
use crate::process::Proc;
use crate::traits::*;

#[derive(PartialEq)]
pub enum Type {
    // attached to the process with 'attach' command
    Attach,

    // spawned a program with 'run' command
    Spawn,

    // Unknown type
    Unknown,
}

pub struct Session {
    pub proc: Proc,

    // Elf object
    pub elf: Option<elf::File>,

    // Path to ELF file
    pub path: Option<path::PathBuf>,

    pub attach_type: Type,
}

impl Session {
    pub fn new() -> Self {
        Session {
            proc: Proc::new(),
            path: None,
            elf:  None,
            attach_type: Type::Unknown,
        }
    }

    pub fn mut_proc(&mut self) -> &mut Proc {
        &mut self.proc
    }

    pub fn set_elf(
        &mut self, 
        path: path::PathBuf
    ) -> Result<(), elf::ParseError> {
        self.elf = Some(elf::File::open_path(&path)?);
        self.path = Some(path);
        Ok(())
    }

    pub fn set_target(&mut self, target: i32) -> Result<i32, ()> {
        self.proc.set(target)
    }

    pub fn set_type(&mut self, t: Type) {
        self.attach_type = t;
    }
}

impl Valid for Session {
    fn valid(&self) -> bool {
        self.proc.valid()
    }
}
