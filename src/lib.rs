/// Los registros de 8bits la CPU
#[derive(PartialEq, Eq)]
#[repr(u8)]
pub enum Reg {
    Invalid = 0,
    A,
    B,
    C,
    D,
    E,
    F,
    H,
    L,
}

/// Los posibles conjuntos de registros usados como contenedor de una dirección
/// Los casos `HLPlus` y `HLMinus` son especiales ya que añaden 1 a la dirección
#[derive(PartialEq, Eq)]
#[repr(u8)]
pub enum RegAddr {
    HL,
    HLPlus,
    HLMinus,
    BC,
    DE,
    Invalid
} 

#[derive(PartialEq, Eq)]
pub enum Instr {
    /// Nop :d
    Nop,

    /// Finalizar la ejecución de la máquina
    Halt,

    /// LD (loads)
    LdRegReg { src: Reg, dest: Reg },
    LdRegImm { src: u8, dest: Reg },
    LdRegMem { src: Reg, dest: RegAddr },
    LdMemReg { src: RegAddr, dest: Reg },
}

pub struct Cpu {
    registers: [u8; 8],
    pc: u16
}

const SRC_TABLE: &[u8] = &[
    0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 1, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
    2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3,
    4, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 5, 5,
    6, 6, 6, 6, 6, 6, 6, 6, 7, 7, 7, 7, 7, 7, 7, 7,
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

const DST_TABLE: &[u8] = &[
    0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 1, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0,
    2, 3, 4, 5, 6, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3,
    2, 3, 4, 5, 6, 4, 4, 4, 5, 5, 5, 5, 5, 5, 5, 5,
    2, 3, 4, 5, 6, 6, 6, 6, 7, 7, 7, 7, 7, 7, 7, 7,
    2, 3, 4, 5, 6, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1,
    0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

impl Cpu {
    pub fn new() -> Self {
        Self {
            registers: [0; 8],
            pc: 0
        }
    }

    pub fn decode(&mut self, instructions: &Vec<u8>) -> Option<Instr> {
        // Extraer el opcode y extraer por separado los primeros y últimos 4 bits
        // que representan la fila y la columna en la matriz de instrucciones
        let opcode = instructions[self.pc as usize];
        let row = opcode >> 4;
        let col = opcode & 0xF;
        let fst_half  = col < 0x8;

        // Avanzar el PC
        self.pc += 1;

        // Discernir la instrucción según el rango que se encuentra
        match opcode {
            0x00 => return Some(Instr::Nop),

            // Cuarta a séptima fila
            0x40..=0x7F => {
                // Detectar temprano la instrucción halt
                if opcode == 0x76 {
                    return Some(Instr::Halt);
                }

                // Según la fila y la columna se puede extraer los registros de 
                // origen y destino
                let reg_src = match col {
                    0x7 | 0xF => Reg::A,
                    0x0 | 0x8 => Reg::B,
                    0x1 | 0x9 => Reg::C,
                    0x2 | 0xA => Reg::D,
                    0x3 | 0xB => Reg::E,
                    0x4 | 0xC => Reg::H,
                    0x5 | 0xD => Reg::L,
                    _ => Reg::Invalid
                };
                let reg_dst = match row {
                    0x4 if fst_half => Reg::B,
                    0x4 => Reg::C,
                    0x5 if fst_half => Reg::D,
                    0x5 => Reg::E,
                    0x6 if fst_half => Reg::H,
                    0x6 => Reg::L,
                    0x7 if !fst_half => Reg::A,
                    _ => Reg::Invalid
                };
                
                // En este punto si ya hemos extraido dos registros válidos ya
                // Tenemos instrucción
                if reg_src != Reg::Invalid && reg_dst != Reg::Invalid {
                    return Some(Instr::LdRegReg { 
                        src: reg_src,
                        dest: reg_dst 
                    });
                }

                // Recoger datos sobre los LD con immediates
                let mut reg_addr = RegAddr::Invalid;
                let mut reg_imm = Reg::Invalid;
                match col {
                    0x6 => match row {
                        0x0 => reg_imm = Reg::B,
                        0x1 => reg_imm = Reg::D,
                        0x2 => reg_imm = Reg::H,
                        0x3 => {
                            let imm = instructions[self.pc as usize];
                            return Some(Instr::LdRegImm {
                                dest: RegAddr::HL,
                                src: imm
                            });
                        },
                        0x4..=0x6 => reg_addr = RegAddr::HL,
                        _ => {}
                    },
                    0xE => match row {
                        0x0 => reg_imm = Reg::B,
                        0x1 => reg_imm = Reg::D,
                        0x2 => reg_imm = Reg::H,
                        0x4..=0x7 => reg_addr = RegAddr::HL,
                        _ => {}
                    }
                    _ => {}
                }

                // En este punto si tenemos algun 
                if reg_imm != Reg::Invalid {
                    let imm = instructions[self.pc.checked_add(1)? as usize];
                    return Some(Instr::LdRegImm { 
                        src: imm, 
                        dest: reg_imm 
                    });
                }
            }

            _ => {
                todo!()
            }
        };

        None
    }

    pub fn write_reg(&mut self, reg: Reg, value: u8) {
        if reg == Reg::Invalid {
            panic!("Cannot write into register Invalid");
        }

        self.registers[reg as usize] = value;
    }
}
