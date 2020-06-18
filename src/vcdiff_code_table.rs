use std::fmt;

#[derive(Debug, Copy, Clone)]
pub enum InstructionType {
    Add,
    Run,
    Copy,
}

#[derive(Debug, Copy, Clone)]
pub struct Instruction {
    pub typ: InstructionType,
    pub size: u8,
    pub mode: u8,
}

pub struct CodeTable {
    pub entries: [(Instruction, Option<Instruction>); 256],
}


impl fmt::Debug for CodeTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      write!(f, "hi")
    }
}

impl CodeTable {
    pub fn default() -> CodeTable {
        let mut vec = [(
            Instruction {
                typ: InstructionType::Add,
                size: 0,
                mode: 0,
            },
            None,
        ); 256];
        let mut idx = 0;
        vec[idx].0 = Instruction {
            typ: InstructionType::Run,
            size: 0,
            mode: 0,
        };
        idx += 1;
        for size in 0..18 {
            vec[idx].0 = Instruction {
                typ: InstructionType::Add,
                size: size,
                mode: 0,
            };
            idx += 1;
        }

        // Entries 19-162
        for mode in 0..9 {
            vec[idx].0 = Instruction {
                typ: InstructionType::Copy,
                size: 0,
                mode: mode,
            };
            idx += 1;
            for size in 4..19 {
                vec[idx].0 = Instruction {
                    typ: InstructionType::Copy,
                    size: size,
                    mode: mode,
                };
                idx += 1;
            }
        }

        // Entries 163-234
        for mode in 0..6 {
            for add_size in 1..5 {
                for copy_size in 4..7 {
                    vec[idx] = (
                        Instruction {
                            typ: InstructionType::Add,
                            size: add_size,
                            mode: 0,
                        },
                        Some(Instruction {
                            typ: InstructionType::Copy,
                            size: copy_size,
                            mode: mode,
                        }),
                    );
                    idx += 1;
                }
            }
        }

        // Entries 235-246
        for mode in 6..9 {
            for add_size in 1..5 {
                vec[idx] = (
                    Instruction {
                        typ: InstructionType::Add,
                        size: add_size,
                        mode: 0,
                    },
                    Some(Instruction {
                        typ: InstructionType::Copy,
                        size: 4,
                        mode: mode,
                    }),
                );
                idx += 1;
            }
        }

        // Entries 247-255
        for mode in 0..9 {
            vec[idx] = (
                Instruction {
                    typ: InstructionType::Copy,
                    size: 4,
                    mode: mode,
                },
                Some(Instruction {
                    typ: InstructionType::Add,
                    size: 1,
                    mode: 0,
                }),
            );
            idx += 1;
        }

        CodeTable { entries: vec }
    }
}
