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
    pub fn decode(bytes: &[u8]) -> Option<CodeTable> {
        if bytes.len() != 256 * 3 * 2 {
            return None;
        }

        let res = (|| -> Result<CodeTable, u32> {
            let mut vec = [(
                Instruction {
                    typ: InstructionType::Add,
                    size: 0,
                    mode: 0,
                },
                None,
            ); 256];
            for i in 0..256 {
                vec[i].0 = Instruction {
                    typ: match bytes[i] {
                        1 => Ok(InstructionType::Add),
                        2 => Ok(InstructionType::Run),
                        3 => Ok(InstructionType::Copy),
                        _ => Err(0u32),
                    }?,
                    size: bytes[i + 512],
                    mode: bytes[i + 1024],
                };
                vec[i].1 = match bytes[i + 256] {
                    0 => Ok(None),
                    1 => Ok(Some(InstructionType::Add)),
                    2 => Ok(Some(InstructionType::Run)),
                    3 => Ok(Some(InstructionType::Copy)),
                    _ => Err(1u32),
                }?.map(|typ| Instruction {
                    typ: typ,
                    size: bytes[i + 256 + 512],
                    mode: bytes[i + 256 + 1024],
                });
            }
            Ok(CodeTable { entries: vec })
        })();

        match res {
            Ok(code_table) => Some(code_table),
            Err(n) => None,
        }
    }

    pub fn encode(&self) -> [u8; 256 * 3 * 2] {
        let mut ret = [0u8; 256 * 3 * 2];

        for i in 0..256 {
            let e = self.entries[i];
            let inst0 = encode_inst(e.0);
            let inst1 = e.1.map_or((0, 0, 0), |inst| encode_inst(inst));
            ret[i] = inst0.0;
            ret[i + 256] = inst1.0;
            ret[i + 512] = inst0.1;
            ret[i + 768] = inst1.1;
            ret[i + 512] = inst0.2;
            ret[i + 768] = inst1.2;
        }

        fn encode_inst(inst: Instruction) -> (u8, u8, u8) {
            (
                match inst.typ {
                    InstructionType::Add => 1,
                    InstructionType::Run => 2,
                    InstructionType::Copy => 3,
                },
                inst.size,
                inst.mode,
            )
        }

        ret
    }

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
