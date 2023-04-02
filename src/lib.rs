/// Los registros de 8bits la CPU
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    SP
}

impl Reg {
    // Constantes usadas solo por claridad para cuando accedo a los registros
    // en su forma ancha (dos unidos)
    pub const DE: Self = Reg::D;
    pub const BC: Self = Reg::B;
    pub const HL: Self = Reg::H;

    pub fn from_u8(value: u8) -> Self {
        debug_assert!(value <= 9);
        unsafe { std::mem::transmute::<u8, Self>(value) }
    }
}

/// Los posibles conjuntos de registros usados como contenedor de una dirección
/// Los casos `HLPlus` y `HLMinus` son especiales ya que añaden 1 a la dirección
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RegAddr {
    Invalid = 0,
    HL = 10,
    HLPlus,
    HLMinus,
    BC,
    DE,
}

impl RegAddr {
    pub fn from_u8(value: u8) -> Self {
        debug_assert!((value >= 10 && value <= 14) || value == 0);
        unsafe { std::mem::transmute::<u8, Self>(value) }
    }
}

#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum InstrKind {
    /// Nop
    Nop = 0,

    /// Control
    Halt = 1,

    /// Loads
    LdRegReg = 2,
    LdRegImm = 3,
    LdRegMem = 4,
    LdMemReg = 5,
    LdMemHLImm = 6,
    
    /// Arithmetic/logical
    AddRegReg = 7,
    AddRegImm = 8,
    AddRegMem = 9,
    AddWRegWReg = 10,
    AddWRegImm = 11,
}

impl InstrKind {
    pub fn from_u8(value: u8) -> Self {
        debug_assert!(value <= 10);
        unsafe { std::mem::transmute::<u8, Self>(value) }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Instr {
    /// Nop :d
    Nop,                                     

    /// Finalizar la ejecución de la máquina
    Halt,

    /// LD (loads)
    LdRegReg { src: Reg,     dst: Reg },
    LdRegImm { src: u8,      dst: Reg },
    LdRegMem { src: Reg,     dst: RegAddr },
    LdMemReg { src: RegAddr, dst: Reg },
    // Es la única operación de carga MemImm, por lo que la hardcodeo
    LdMemHLImm,

    /// Arithmetic/logical operations
    AddRegReg { src: Reg, dst: Reg },
    AddRegImm { src: u8,  dst: Reg },
    AddRegMem { src: RegAddr, dst: Reg },
    AddWRegWReg { src: Reg, dst: Reg },
}

#[derive(Debug)]
pub struct Cpu {
    /// Hay 8, registros de 8-bits, 3 registros de 16-bits que son las unión de
    /// 2 registros de 8-bits BC, DE y HL, además del Stack Pointer (SP) que es
    /// el único registro de 16-bits independiente, lo incluyo en este array en
    /// vez de crear un campo separado como para PC ya que este es usado 
    /// bastante como operando en las intrucciones y me facilita su decode
    registers: [u8; 10],

    /// Program counter
    pc: u16
}

/// Esta tabla se usa para discernir el tipo de instrucción `InstrKind` que 
/// luego se convierte a `Instr` accediendo a las otras tablas
const INST_KIND_TABLE: &[u8] = &[
    0, 0, 4, 0, 0, 0, 3, 0, 0,10, 5, 0, 0, 0, 3, 0,
    0, 0, 4, 0, 0, 0, 3, 0, 0,10, 5, 0, 0, 0, 3, 0,
    0, 0, 4, 0, 0, 0, 3, 0, 0,10, 5, 0, 0, 0, 3, 0,
    0, 0, 4, 0, 0, 0, 3, 0, 0,10, 5, 0, 0, 0, 3, 0,
    2, 2, 2, 2, 2, 2, 5, 2, 2, 2, 2, 2, 2, 2, 5, 2,
    2, 2, 2, 2, 2, 2, 5, 2, 2, 2, 2, 2, 2, 2, 5, 2,
    2, 2, 2, 2, 2, 2, 5, 2, 2, 2, 2, 2, 2, 2, 5, 2,
    4, 4, 4, 4, 4, 4, 1, 4, 2, 2, 2, 2, 2, 2, 5, 2,
    7, 7, 7, 7, 7, 7, 9, 7, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 9, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

/// Tabla usada para discernir el operando de entrada de la instrucción, sus
/// valores son convertibles directamente a los enums `Reg` y `RegMem`, en 
/// release la conversión se hace sin comprobaciones
const SRC_TABLE: &[u8] = &[
    0, 0, 1, 0, 0, 0, 0, 0, 0, 2,13, 0, 0, 0, 0, 0,
    0, 0, 1, 0, 0, 0, 0, 0, 0, 4,14, 0, 0, 0, 0, 0,
    0, 0, 1, 0, 0, 0, 0, 0, 0, 7,11, 0, 0, 0, 0, 0,
    0, 0, 1, 0, 0, 0, 0, 0, 0, 9,12, 0, 0, 0, 0, 0,
    2, 3, 4, 5, 7, 8,10, 1, 2, 3, 4, 5, 7, 8,10, 1,
    2, 3, 4, 5, 7, 8,10, 1, 2, 3, 4, 5, 7, 8,10, 1,
    2, 3, 4, 5, 7, 8,10, 1, 2, 3, 4, 5, 7, 8,10, 1,
    2, 3, 4, 5, 7, 8, 0, 1, 2, 3, 4, 5, 7, 8,10, 1,
    2, 3, 4, 5, 7, 8,10, 1, 2, 3, 4, 5, 7, 8,10, 1,
    2, 3, 4, 5, 7, 8,10, 1, 2, 3, 4, 5, 7, 8,10, 1,
    2, 3, 4, 5, 7, 8,10, 1, 2, 3, 4, 5, 7, 8,10, 1,
    2, 3, 4, 5, 7, 8,10, 1, 2, 3, 4, 5, 7, 8,10, 1,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

/// Tabla usada para discernir el operando destino
const DST_TABLE: &[u8] = &[
    0, 0,12, 0, 0, 0, 2, 0, 0, 7, 1, 0, 0, 3, 0, 0,
    0, 0,13, 0, 0, 0, 4, 0, 0, 7, 1, 0, 0, 5, 0, 0,
    0, 0,10, 0, 0, 0, 7, 0, 0, 7, 1, 0, 0, 8, 0, 0,
    0, 0,11, 0, 0, 0, 9, 0, 0, 7, 1, 0, 0, 1, 0, 0,
    2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3,
    4, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 5, 5,
    6, 6, 6, 6, 6, 6, 6, 6, 7, 7, 7, 7, 7, 7, 7, 7,
    9, 9, 9, 9, 9, 9, 0, 9, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

/// Detiene la ejecución del programa (sleep) durante una cantidad de tiempo
/// dependiente de qué tenga la cpu configurado como un tick
macro_rules! tick {
    ($self:expr, $n:expr) => {
        // TODO
    }
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            registers: [0; 10],
            pc: 0
        }
    }

    // TODO: Las instrucciones se deberán leer de la MMU y no pasarlas como un
    // slice como si se supiera exactamente cuales valores en memoria son o no
    // realmente instrucciones
    pub fn decode(&mut self, instructions: &[u8]) -> Option<Instr> {
        // Extraer el opcode y extraer por separado los primeros y últimos 4 bits
        // que representan la fila y la columna en la matriz de instrucciones
        let opcode = instructions[self.pc as usize];

        // Avanzar el PC
        self.pc += 1;

        // macro_rules! decode_reg_reg {
        //     ($variant:ident) => {
        //         // Extraer registros de origen y destino
        //     }
        // }
        
        match InstrKind::from_u8(INST_KIND_TABLE[opcode as usize]) {
            InstrKind::Halt => Some(Instr::Halt),
            InstrKind::LdRegReg => {
                // Extract source and destination registers
                let src = SRC_TABLE[opcode as usize];
                let dst = DST_TABLE[opcode as usize];

                assert!(src != 0);
                assert!(dst != 0);

                let src = Reg::from_u8(src);
                let dst = Reg::from_u8(dst);

                Some(Instr::LdRegReg { src, dst })
            },
            InstrKind::LdRegImm => {
                // Extraer immediate
                let imm = instructions[self.pc as usize];
                self.pc += 1;
                
                // Extraer registro destino
                let dst = DST_TABLE[opcode as usize];

                assert!(dst != 0);

                let dst = Reg::from_u8(dst);

                Some(Instr::LdRegImm { src: imm, dst })
            },
            InstrKind::LdRegMem => {
                // Extract source register and destination memory address as
                // register
                let src = SRC_TABLE[opcode as usize];
                let dst = DST_TABLE[opcode as usize];

                assert!(src != 0);
                assert!(dst != 0);

                let src = Reg::from_u8(src);
                let dst = RegAddr::from_u8(dst);

                Some(Instr::LdRegMem { src, dst })
            },
            InstrKind::LdMemReg => {
                // Extract source memory address as register and destination 
                // register
                let src = SRC_TABLE[opcode as usize];
                let dst = DST_TABLE[opcode as usize];

                assert!(src != 0);
                assert!(dst != 0);

                let src = RegAddr::from_u8(src);
                let dst = Reg::from_u8(dst);

                Some(Instr::LdMemReg { src, dst })
            },
            InstrKind::AddRegReg => {
                // Extract source and destination registers
                let src = SRC_TABLE[opcode as usize];
                let dst = DST_TABLE[opcode as usize];

                assert!(src != 0);
                assert!(dst != 0);

                let src = Reg::from_u8(src);
                let dst = Reg::from_u8(dst);

                Some(Instr::AddRegReg { src, dst })
            }
            _ => { None }
        }
    }

    /// Escribir en un registro de 8-bits
    #[inline]
    pub fn write_reg(&mut self, reg: Reg, value: u8) {
        if cfg!(debug_assertions) && reg == Reg::Invalid {
            panic!("Cannot write into register Invalid");
        }

        self.registers[reg as usize - 1] = value;
    }

    /// Leer de un registro de 8-bits
    #[inline]
    pub fn read_reg(&self, reg: Reg) -> u8 {
        if cfg!(debug_assertions) && reg == Reg::Invalid {
            panic!("Cannot read into register Invalid");
        }

        self.registers[reg as usize - 1]
    }

    /// Leer de dos registros que forman un valor de 16-bits, solo se puede
    /// hacer sobre los registros ampliados BC, DE y HL
    #[inline]
    pub fn read_widereg(&self, reg: Reg) -> u16 {
        if cfg!(debug_assertions) && 
                matches!(reg, 
                    Reg::Invalid | Reg::A | Reg::C | Reg::E | Reg::F | Reg::L) 
        {
            panic!("Cannot wide read into this register {:?}", reg);
        }
        
        let reg_range = (reg as usize - 1)..=reg as usize;
        u16::from_ne_bytes(self.registers[reg_range].try_into().unwrap())
    }

    /// Escribir en dos registros que forman un valor de 16-bits, solo se puede
    /// hacer sobre los registros ampliados BC, DE y HL
    #[inline]
    pub fn write_widereg(&mut self, reg: Reg, value: u16) {
        if cfg!(debug_assertions) && 
                matches!(reg, 
                    Reg::Invalid | Reg::A | Reg::C | Reg::E | Reg::F | Reg::L) 
        {
            panic!("Cannot wide write into this register {:?}", reg);
        }

        let [l, h] = u16::to_ne_bytes(value);
        self.registers[reg as usize - 1] = l;
        self.registers[reg as usize] = h;
    }

    // TODO: A esta función habrá que pasarle la MMU
    pub fn execute(&mut self, instructions: &[u8]) -> Option<()> {
        // Hacer decode de la instrucción a ejecutar
        let instr = self.decode(instructions)?;

        // Realizar la ejecución según instrucción
        match instr {
            Instr::Nop => {},
            Instr::Halt => { todo!() },
            Instr::LdRegReg { src, dst } => {
                tick!(self, 4);
                self.write_reg(dst, self.read_reg(src));
            },
            Instr::LdRegImm { src, dst } => {
                tick!(self, 8);
                self.write_reg(dst, src);
            },
            Instr::LdRegMem { .. } => todo!(),
            Instr::LdMemReg { .. } => todo!(),
            Instr::LdMemHLImm => todo!(),
            Instr::AddRegReg { src, dst } => {
                tick!(self, 4);
                self.write_reg(dst, self.read_reg(src) + self.read_reg(dst));
            }
            _ => todo!()
        }

        Some(())
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let example_program = &[
            0x40, 0x50, 0x46
        ];

        let mut cpu = Cpu::new();
        assert_eq!(
            cpu.decode(example_program.as_slice()), 
            Some(Instr::LdRegReg {
                src: Reg::B,
                dst: Reg::B
            })
        );
        assert_eq!(
            cpu.decode(example_program.as_slice()), 
            Some(Instr::LdRegReg {
                src: Reg::B,
                dst: Reg::D
            })
        );
        assert_eq!(
            cpu.decode(example_program.as_slice()), 
            Some(Instr::LdMemReg {
                src: RegAddr::HL,
                dst: Reg::B
            })
        );
    }
}
