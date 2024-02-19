mod mmu;

use crate::mmu::Mmu;

/// Los registros de 8bits la CPU
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Reg {
    Invalid = 0,
    A = 1,
    F = 2,
    B = 3,
    C = 4,
    D = 5,
    E = 6,
    H = 7,
    L = 8,
    SP = 9
}

impl Reg {
    // Constantes usadas solo por claridad para cuando accedo a los registros
    // en su forma ancha (dos unidos)
    pub const AF: Self = Reg::A;
    pub const BC: Self = Reg::B;
    pub const DE: Self = Reg::D;
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
        debug_assert!((value >= 10 && value <= 15) || value == 0);
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

    /// Basic Loads
    LdRegReg = 2,
    LdRegImm = 3,
    LdRegMem = 4,
    LdMemReg = 5,
    LdMemHLImm = 6,
    
    /// Arithmetic/logical
    AddRegReg = 7,
    AddRegImm = 8,
    AddMemReg = 9,
    AddWRegWReg = 10,
    AddWRegImm = 11,

    AdcRegReg = 12,
    AdcRegImm = 13,
    AdcMemReg = 14,

    SubReg = 15,
    SubImm = 16,
    SubMem = 17,

    SbcReg = 18,
    SbcImm = 19,
    SbcMem = 20,

    AndReg = 21,
    AndImm = 22,
    AndMem = 23,

    XorReg = 24,
    XorImm = 25,
    XorMem = 26,

    OrReg = 27,
    OrImm = 28,
    OrMem = 29,

    IncReg = 30,
    IncWReg = 31,
    IncMem = 32,
    
    DecReg = 33,
    DecWReg = 34,
    DecMem = 35,

    CpReg = 36,
    CpImm = 37,
    CpMem = 38,

    /// Advanced loads / stack
    LdWRegImm = 40,
    LdMemImmReg = 41,
    Push = 42,
    Pop = 43,
    AddSPImm = 44,

    /// Jumps
    JPImm = 45,
    JPCond = 46,
    JPReg = 47,
    JRelImm = 48,
    JRelCond = 49,
    Rst = 50,

    /// Bit manipulation
    RlcA = 51,
    RlA = 52,
    RrcA = 53,
    RrA = 54,

    /// Prefixed instrucctions 0xCB
    RlcReg = 55,
    RlcMem = 56,
    RrcReg = 57,
    RrcMem = 58,
    RlReg = 59,
    RlMem = 60,
    RrReg = 61,
    RrMem = 62,
    SlaReg = 63,
    SlaMem = 64,
    SraReg = 65,
    SraMem = 66,
    SwapReg = 67,
    SwapMem = 68,
    SrlReg = 69,
    SrlMem = 70,
    BitReg = 71,
    BitMem = 72,
    ResReg = 73,
    ResMem = 74,
    SetReg = 75,
    SetMem = 76,

    /// Calls and returns
    Ret,
    RetCond,
    Reti,
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
    AddMemReg { src: RegAddr, dst: Reg },
    AddWRegWReg { src: Reg, dst: Reg },
    AddWRegImm { src: u8, dst: Reg },

    AdcRegReg { src: Reg, dst: Reg },
    AdcRegImm { src: u8, dst: Reg },
    AdcMemReg { src: RegAddr, dst: Reg },

    SubReg { src: Reg },
    SubImm { src: u8 },
    SubMem { src: RegAddr },

    SbcReg { src: Reg },
    SbcImm { src: u8 },
    SbcMem { src: RegAddr },

    AndReg { src: Reg },
    AndImm { src: u8 },
    AndMem { src: RegAddr },

    OrReg { src: Reg },
    OrImm { src: u8 },
    OrMem { src: RegAddr },
     

    IncReg { dst: Reg },
    IncWReg { dst: Reg },
    IncMem { dst: RegAddr },
    
    DecReg { dst: Reg },
    DecWReg { dst: Reg },
    DecMem { dst: RegAddr },

    CpReg { src: Reg },
    CpImm { src: u8 },
    CpMem { src: RegAddr },

    LdWRegImm { src: u16, dst: Reg },
    LdMemImmReg { src: Reg, dst: u16 },
    Push { src: Reg },
    Pop { dst: Reg },

    JPImm { addr: u16 },
    JPCond { cond: u8, addr: u16 },
    JPReg { src: Reg },
    JRelImm { offset: u8 },
    JRelCond { cond: u8, offset: u8 },
    Rst { addr: u8 },

    RlcReg { reg: Reg },
    RlcMem { reg: RegAddr },
    RrcReg { reg: Reg },
    RrcMem { reg: RegAddr },
    RlReg { reg: Reg },
    RlMem { reg: RegAddr },
    RrReg { reg: Reg },
    RrMem { reg: RegAddr },
    SlaReg { reg: Reg },
    SlaMem { reg: RegAddr },
    SraReg { reg: Reg },
    SraMem { reg: RegAddr },
    SwapReg { reg: Reg }, 
    SwapMem { reg: RegAddr }, 
    SrlReg { reg: Reg }, 
    SrlMem { reg: RegAddr },
    BitReg { reg: Reg, bit: u8 },
    BitMem { reg: RegAddr, bit: u8 },
    ResReg { reg: Reg, bit: u8 },
    ResMem { reg: RegAddr, bit: u8 },
    SetReg { reg: Reg, bit: u8 },
    SetMem { reg: RegAddr, bit: u8 },

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

/// Zero Flag: Se activa cuando el resultado de la última operación matemática
/// fue un 0 o CP sobre dos valores retorna 0
const FLAG_Z: u8 = 1 << 7;

/// Substract Flag: se activa si la última operación realizada fue un SUB
const FLAG_N: u8 = 1 << 6;

/// Half Carry Flag: se activa cuando se hace overflow en el grupo inferior de
/// una operación arimétrica de 8-bits, es decir que hay carry a partir del
/// bit 3
const FLAG_H: u8 = 1 << 5;

/// Carry Flag: se activa cuando la operación matemática hace overflow o cuando
/// el registro A es el menor valor al ejecutar la instrucción CP
const FLAG_C: u8 = 1 << 4;

/// Esta tabla se usa para discernir el tipo de instrucción `InstrKind` que 
/// luego se convierte a `Instr` accediendo a las otras tablas
const INST_KIND_TABLE: &[u8] = &[
    0,40, 4, 0, 0, 0, 3, 0, 0,10, 5, 0, 0, 0, 3, 0,
    0,40, 4, 0, 0, 0, 3, 0,48,10, 5, 0, 0, 0, 3, 0,
   49,40, 4, 0, 0, 0, 3, 0,49,10, 5, 0, 0, 0, 3, 0,
   49,40, 4, 0, 0, 0, 3, 0,49,10, 5, 0, 0, 0, 3, 0,
    2, 2, 2, 2, 2, 2, 5, 2, 2, 2, 2, 2, 2, 2, 5, 2,
    2, 2, 2, 2, 2, 2, 5, 2, 2, 2, 2, 2, 2, 2, 5, 2,
    2, 2, 2, 2, 2, 2, 5, 2, 2, 2, 2, 2, 2, 2, 5, 2,
    4, 4, 4, 4, 4, 4, 1, 4, 2, 2, 2, 2, 2, 2, 5, 2,
    7, 7, 7, 7, 7, 7, 9, 7,12,12,12,12,12,12,14,12,
   15,15,15,15,15,15,17,15,18,18,18,18,18,18,18,18,
   21,21,21,21,21,21,23,21,24,24,24,24,24,24,24,24,
   27,27,27,27,27,27,27,28, 0, 0, 0, 0, 0, 0, 0, 0,
    0,43, 0,46,45,42, 9, 0, 0, 0, 0,46, 0, 0,13, 0,
    0,43, 0,46, 0,42,16, 0, 0, 0, 0,46, 0, 0,19, 0,
    0,43, 0, 0, 0,42,22, 0,11, 0,47, 0, 0, 0,25, 0,
    0,43, 0, 0, 0,42,28, 0, 0, 0,10, 0, 0, 0, 0, 0,
];

const NZ: u8 = FLAG_N | FLAG_Z;
const NC: u8 = FLAG_N | FLAG_C;
const Z:  u8 = FLAG_Z;
const C:  u8 = FLAG_C;

/// Tabla usada para discernir el operando de entrada de la instrucción, sus
/// valores son convertibles directamente a los enums `Reg` y `RegMem`, en 
/// release la conversión se hace sin comprobaciones
const SRC_TABLE: &[u8] = &[
    0, 0, 1, 3, 3, 3, 0, 0, 9, 3,13, 3, 4, 3, 0, 0,
    0, 0, 1, 5, 5, 5, 0, 0, 0, 5,14, 5, 6, 5, 0, 0,
   NZ, 0, 1, 7, 7, 7, 0, 0, Z, 7,11, 7, 8, 8, 0, 0,
   NC, 0, 1, 9,10,10, 0, 0, C, 9,12, 9, 1, 1, 0, 0,
    3, 3, 4, 6, 7, 8,10, 1, 3, 4, 5, 6, 7, 8,10, 1,
    3, 4, 5, 6, 7, 8,10, 1, 3, 4, 5, 6, 7, 8,10, 1,
    3, 4, 5, 6, 7, 8,10, 1, 3, 4, 5, 6, 7, 8,10, 1,
    3, 4, 5, 6, 7, 8, 0, 1, 3, 4, 5, 6, 7, 8,10, 1,
    3, 4, 5, 6, 7, 8,10, 1, 3, 4, 5, 6, 7, 8,10, 1,
    3, 4, 5, 6, 7, 8,10, 1, 3, 4, 5, 6, 7, 8,10, 1,
    3, 4, 5, 6, 7, 8,10, 1, 3, 4, 5, 6, 7, 8,10, 1,
    3, 4, 5, 6, 7, 8,10, 1, 3, 4, 5, 6, 7, 8,10, 1,
   NZ, 3,NZ, 0, 0, 3, 0, 0, Z, 0, Z, 0, Z, 0, 0, 0,
   NC, 5,NC, 0, 0, 5, 0, 0, C, 0, C, 0, C, 0, 0, 0,
    0, 7, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 1, 0, 1, 0, 1, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0,
];

/// Tabla usada para discernir el operando destino
const DST_TABLE: &[u8] = &[
    0, 3,12, 0, 0, 0, 3, 0, 0, 7, 1, 0, 0, 4, 0, 0,
    0, 5,13, 0, 0, 0, 5, 0, 0, 7, 1, 0, 0, 6, 0, 0,
    0, 7,10, 0, 0, 0, 7, 0, 0, 7, 1, 0, 0, 8, 0, 0,
    0, 9,11, 0, 0, 0, 9, 0, 0, 7, 1, 0, 0, 1, 0, 0,
    3, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4,
    5, 5, 5, 5, 5, 5, 5, 5, 6, 6, 6, 6, 6, 6, 6, 6,
    7, 7, 7, 7, 7, 7, 7, 7, 8, 8, 8, 8, 8, 8, 8, 8,
   10,10,10,10,10,10, 0,10, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 9, 0, 0, 0, 0, 0,
];

/// Tabla usada para discernir el operando destino
const PREFIX_TABLE: &[u8] = &[
   55,55,55,55,55,55,56,55,57,57,57,57,57,57,58,57,
   59,59,59,59,59,59,60,59,61,61,61,61,61,61,62,61,
   63,63,63,63,63,63,64,63,65,65,65,65,65,65,66,65,
   67,67,67,67,67,67,68,67,69,69,69,69,69,69,70,69,
   71,71,71,71,71,71,71,71,73,73,73,73,73,73,73,73,
   75,75,75,75,75,75,75,75,77,77,77,77,77,77,77,77,
   79,79,79,79,79,79,79,79,81,81,81,81,81,81,81,81,
   83,83,83,83,83,83,83,83,85,85,85,85,85,85,85,85,
   87,87,87,87,87,87,87,87,89,89,89,89,89,89,89,89,
   91,91,91,91,91,91,91,91,93,93,93,93,93,93,93,93,
   95,95,95,95,95,95,95,95,97,97,97,97,97,97,97,97,    
    0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 9, 0, 0, 0, 0, 0,
];

/// Tabla usada para discernir el operando destino
const PREFIX_SRC_TABLE: &[u8] = &[
   0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
   0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
   0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
   0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
   0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1,
   2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3,
   4, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 5, 5,
   6, 6, 6, 6, 6, 6, 6, 6, 7, 7, 7, 7, 7, 7, 7, 7,
   0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1,
   2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3,
   4, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 5, 5,
   6, 6, 6, 6, 6, 6, 6, 6, 7, 7, 7, 7, 7, 7, 7, 7,
   0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1,
   2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3,
   4, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 5, 5,
   6, 6, 6, 6, 6, 6, 6, 6, 7, 7, 7, 7, 7, 7, 7, 7,
];

/// Tabla usada para discernir el operando destino
const PREFIX_DST_TABLE: &[u8] = &[
   3, 4, 5, 6, 7, 8, 10, 1, 3, 4, 5, 6, 7, 8, 10, 1,
   3, 4, 5, 6, 7, 8, 10, 1, 3, 4, 5, 6, 7, 8, 10, 1,
   3, 4, 5, 6, 7, 8, 10, 1, 3, 4, 5, 6, 7, 8, 10, 1,
   3, 4, 5, 6, 7, 8, 10, 1, 3, 4, 5, 6, 7, 8, 10, 1,
   3, 4, 5, 6, 7, 8, 10, 1, 3, 4, 5, 6, 7, 8, 10, 1,
   3, 4, 5, 6, 7, 8, 10, 1, 3, 4, 5, 6, 7, 8, 10, 1,
   3, 4, 5, 6, 7, 8, 10, 1, 3, 4, 5, 6, 7, 8, 10, 1,
   3, 4, 5, 6, 7, 8, 10, 1, 3, 4, 5, 6, 7, 8, 10, 1,
   3, 4, 5, 6, 7, 8, 10, 1, 3, 4, 5, 6, 7, 8, 10, 1,
   3, 4, 5, 6, 7, 8, 10, 1, 3, 4, 5, 6, 7, 8, 10, 1,
   3, 4, 5, 6, 7, 8, 10, 1, 3, 4, 5, 6, 7, 8, 10, 1,
   3, 4, 5, 6, 7, 8, 10, 1, 3, 4, 5, 6, 7, 8, 10, 1,
   3, 4, 5, 6, 7, 8, 10, 1, 3, 4, 5, 6, 7, 8, 10, 1,
   3, 4, 5, 6, 7, 8, 10, 1, 3, 4, 5, 6, 7, 8, 10, 1,
   3, 4, 5, 6, 7, 8, 10, 1, 3, 4, 5, 6, 7, 8, 10, 1,
   3, 4, 5, 6, 7, 8, 10, 1, 3, 4, 5, 6, 7, 8, 10, 1,
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

        // Macros útiles para no repetir código en el decode
        macro_rules! decode_reg {
            ($loc:ident, $variant:ident) => {{
                // Extraer registro
                let $loc = SRC_TABLE[opcode as usize];

                assert!($loc != 0);

                let $loc = Reg::from_u8($loc);

                Some(Instr::$variant { $loc })
            }};
        }

        macro_rules! decode_imm {
            ($loc:ident, $variant:ident) => {{
                // Extraer immediate
                let imm = instructions[self.pc as usize];
                self.pc += 1;

                Some(Instr::$variant { $loc: imm })
            }};
        }

        macro_rules! decode_mem {
            ($loc:ident, $variant:ident) => {{
                // Extraer registro
                let $loc = SRC_TABLE[opcode as usize];

                assert!($loc != 0);

                let $loc = RegAddr::from_u8($loc);

                Some(Instr::$variant { $loc })            
            }};
        }

        macro_rules! decode_reg_reg {
            ($variant:ident) => {{
                // Extraer registros de origen y destino
                let src = SRC_TABLE[opcode as usize];
                let dst = DST_TABLE[opcode as usize];

                assert!(src != 0);
                assert!(dst != 0);

                let src = Reg::from_u8(src);
                let dst = Reg::from_u8(dst);

                Some(Instr::$variant { src, dst })
            }}
        }

        macro_rules! decode_reg_imm {
            ($variant:ident) => {{
                // Extraer immediate
                let imm = instructions[self.pc as usize];
                self.pc += 1;
                
                // Extraer registro destino
                let dst = DST_TABLE[opcode as usize];

                assert!(dst != 0);

                let dst = Reg::from_u8(dst);

                Some(Instr::$variant { src: imm, dst })        
            }}
        }

        macro_rules! decode_reg_mem {
            ($variant:ident) => {{
                // Extraer registro origen y direccion de memoria en registro 
                let src = SRC_TABLE[opcode as usize];
                let dst = DST_TABLE[opcode as usize];

                assert!(src != 0);
                assert!(dst != 0);

                let src = Reg::from_u8(src);
                let dst = RegAddr::from_u8(dst);

                Some(Instr::$variant { src, dst })
            }}
        }

        macro_rules! decode_mem_reg {
            ($variant:ident) => {{
                // Extract source memory address as register and destination 
                // register
                let src = SRC_TABLE[opcode as usize];
                let dst = DST_TABLE[opcode as usize];

                assert!(src != 0);
                assert!(dst != 0);

                let src = RegAddr::from_u8(src);
                let dst = Reg::from_u8(dst);

                Some(Instr::$variant { src, dst })
            }}
        }

        macro_rules! prefix_decode_reg {
            ($loc:ident, $variant:ident) => {{
                // Extraer registro
                let $loc = PREFIX_DST_TABLE[opcode as usize];

                assert!($loc != 0);

                let $loc = Reg::from_u8($loc);

                Some(Instr::$variant { $loc })
            }};
        }

        macro_rules! prefix_decode_mem {
            ($loc:ident, $variant:ident) => {{
                // Extraer registro
                let $loc = PREFIX_DST_TABLE[opcode as usize];

                assert!($loc != 0);

                let $loc = RegAddr::from_u8($loc);

                Some(Instr::$variant { $loc })            
            }};
        }

        macro_rules! prefix_decode_reg_imm {
            ($reg_loc:ident, $imm_loc:ident, $variant:ident) => {{
                // Extraer immediate
                let imm = instructions[self.pc as usize];
                self.pc += 1;
                
                // Extraer registro destino
                let dst = PREFIX_DST_TABLE[opcode as usize];

                assert!(dst != 0);

                let dst = Reg::from_u8(dst);

                Some(Instr::$variant { $imm_loc: imm, $reg_loc: dst })
            }}
        }

        macro_rules! prefix_decode_mem_imm {
            ($mem_loc:ident, $imm_loc:ident, $variant:ident) => {{
                // Extraer immediate
                let imm = instructions[self.pc as usize];
                self.pc += 1;
                
                // Extraer registro como mem destino
                let dst = PREFIX_DST_TABLE[opcode as usize];

                assert!(dst != 0);

                let dst = RegAddr::from_u8(dst);

                Some(Instr::$variant { $imm_loc: imm, $mem_loc: dst })
            }}
        }

        let mut res;

        // Common (unprefixed) instructions
        res = match InstrKind::from_u8(INST_KIND_TABLE[opcode as usize]) {
            InstrKind::Halt => Some(Instr::Halt),
            InstrKind::LdRegReg => decode_reg_reg!(LdRegReg),
            InstrKind::LdRegImm => decode_reg_imm!(LdRegImm),
            InstrKind::LdRegMem => decode_reg_mem!(LdRegMem),
            InstrKind::LdMemReg => decode_mem_reg!(LdMemReg),
            InstrKind::AddRegReg => decode_reg_reg!(AddRegReg),
            InstrKind::AddRegImm => decode_reg_imm!(AddRegImm),
            InstrKind::AddMemReg => decode_mem_reg!(AddMemReg),
            InstrKind::AddWRegWReg => decode_reg_reg!(AddWRegWReg),
            InstrKind::AdcRegReg => decode_reg_reg!(AdcRegReg),
            InstrKind::AdcRegImm => decode_reg_imm!(AdcRegImm),
            InstrKind::AdcMemReg => decode_mem_reg!(AdcMemReg),
            InstrKind::SubReg => decode_reg!(src, SubReg),
            InstrKind::SubImm => decode_imm!(src, SubImm),
            InstrKind::SubMem => decode_mem!(src, SubMem),
            InstrKind::SbcReg => decode_reg!(src, SbcReg),
            InstrKind::SbcImm => decode_imm!(src, SbcImm),
            InstrKind::SbcMem => decode_mem!(src, SbcMem),
            InstrKind::AndReg => decode_reg!(src, AndReg),
            InstrKind::AndImm => decode_imm!(src, AndImm),
            InstrKind::AndMem => decode_mem!(src, AndMem),
            InstrKind::OrReg => decode_reg!(src, OrReg),
            InstrKind::OrImm => decode_imm!(src, OrImm),
            InstrKind::OrMem => decode_mem!(src, OrMem),
            InstrKind::IncReg => decode_reg!(dst, IncReg),
            InstrKind::IncWReg => decode_reg!(dst, IncWReg),
            InstrKind::IncMem => decode_mem!(dst, IncMem),
            InstrKind::DecReg => decode_reg!(dst, DecReg),
            InstrKind::DecWReg => decode_reg!(dst, DecWReg),
            InstrKind::DecMem => decode_mem!(dst, DecMem),
            InstrKind::CpReg => decode_reg!(src, CpReg),
            InstrKind::CpImm => decode_imm!(src, CpImm),
            InstrKind::CpMem => decode_mem!(src, CpMem),
            InstrKind::LdWRegImm => {
                // Extraer immediate
                let immh = instructions[self.pc as usize];
                self.pc += 1;
                let imml = instructions[self.pc as usize];
                self.pc += 1;
                let imm = u16::from_le_bytes([immh, imml]);
                
                // Extraer registro destino
                let dst = DST_TABLE[opcode as usize];

                assert!(dst != 0);

                let dst = Reg::from_u8(dst);

                Some(Instr::LdWRegImm { src: imm, dst })
            },
            InstrKind::LdMemImmReg => {
                // Extraer immediate
                let immh = instructions[self.pc as usize];
                self.pc += 1;
                let imml = instructions[self.pc as usize];
                self.pc += 1;
                let imm = u16::from_le_bytes([immh, imml]);

                // Extraer registro origen
                let src = SRC_TABLE[opcode as usize];

                assert!(src != 0);

                let src = Reg::from_u8(src);

                Some(Instr::LdMemImmReg { src, dst: imm })
            }
            InstrKind::Push => decode_reg!(src, Push),
            InstrKind::Pop  => decode_reg!(dst, Pop),
            InstrKind::JPImm => {
                // Extraer immediate
                let immh = instructions[self.pc as usize];
                self.pc += 1;
                let imml = instructions[self.pc as usize];
                self.pc += 1;
                let imm = u16::from_le_bytes([immh, imml]);

                Some(Instr::JPImm { addr: imm })
            },
            InstrKind::JPCond => {
                // Extraer immediate
                let immh = instructions[self.pc as usize];
                self.pc += 1;
                let imml = instructions[self.pc as usize];
                self.pc += 1;
                let imm = u16::from_le_bytes([immh, imml]);
                
                // Extraer condition
                let cond = SRC_TABLE[opcode as usize];

                assert!(cond != 0);

                Some(Instr::JPCond { cond, addr: imm })
            },
            InstrKind::JPReg => decode_reg!(src, JPReg),
            InstrKind::JRelImm => decode_imm!(offset, JRelImm),
            InstrKind::JRelCond => {
                // Extraer immediate
                let imm = instructions[self.pc as usize];
                self.pc += 1;

                // Extraer condition
                let cond = SRC_TABLE[opcode as usize];

                assert!(cond != 0);

                Some(Instr::JRelCond { cond, offset: imm })
            },

            _ => { None }
        };

        // Prefixed instructions
        res = if res.is_none() && opcode == 0xCB {
            match InstrKind::from_u8(PREFIX_TABLE[opcode as usize]) {
                InstrKind::RlcReg => prefix_decode_reg!(reg, RlcReg),
                InstrKind::RlcMem => prefix_decode_mem!(reg, RlcMem),
                InstrKind::RrcReg => prefix_decode_reg!(reg, RrcReg),
                InstrKind::RrcMem => prefix_decode_mem!(reg, RrcMem),
                InstrKind::RlReg => prefix_decode_reg!(reg, RlReg),
                InstrKind::RlMem => prefix_decode_mem!(reg, RlMem),
                InstrKind::RrReg => prefix_decode_reg!(reg, RrReg),
                InstrKind::RrMem => prefix_decode_mem!(reg, RrMem),
                InstrKind::SlaReg => prefix_decode_reg!(reg, SlaReg),
                InstrKind::SlaMem => prefix_decode_mem!(reg, SlaMem),
                InstrKind::SraReg => prefix_decode_reg!(reg, SraReg),
                InstrKind::SraMem => prefix_decode_mem!(reg, SraMem),
                InstrKind::BitReg => prefix_decode_reg_imm!(reg, bit, BitReg),
                InstrKind::BitMem => prefix_decode_mem_imm!(reg, bit, BitMem),
                InstrKind::ResReg => prefix_decode_reg_imm!(reg, bit, ResReg),
                InstrKind::ResMem => prefix_decode_mem_imm!(reg, bit, ResMem),
                InstrKind::SetReg => prefix_decode_reg_imm!(reg, bit, SetReg),
                InstrKind::SetMem => prefix_decode_mem_imm!(reg, bit, SetMem),
                _ => None,
            }
        } else {
           unreachable!()
        };
        
        res
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
        u16::from_le_bytes(self.registers[reg_range].try_into().unwrap())
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

        let [l, h] = u16::to_le_bytes(value);
        self.registers[reg as usize - 1] = l;
        self.registers[reg as usize] = h;
    }

    /// Sumar dos valores de 8-bits de la alu    
    // TODO: Maybe on the future creating a trait that joins the normal and
    // wide word operations under it will simplify code
    #[inline]
    fn alu_add(&mut self, a: u8, b: u8) -> u8 {
        // Realizar la operación y decidir que flags se activan
        let (res, carry) = a.overflowing_add(b);
        let half_carry = res >> 4 != 0;
        let zero = res == 0;

        // Crear el u8 de flags de la operación
        let mut flags = 0;
        if carry {
            flags |= FLAG_C;
        }
        if half_carry {
            flags |= FLAG_H;
        }
        if zero {
            flags |= FLAG_Z;
        }
        self.write_reg(Reg::F, flags);

        res
    }

    /// Sumar dos valores de 16-bits en la alu
    #[inline]
    fn alu_wideadd(&mut self, a: u16, b: u16) -> u16 {
        // Realizar la operación y decidir que flags se activan
        let (res, carry) = a.overflowing_add(b);
        let half_carry = res >> 12 != 0;
        let zero = res == 0;

        // Crear el u8 de flags de la operación
        let mut flags = 0;
        if carry {
            flags |= FLAG_C;
        }
        if half_carry {
            flags |= FLAG_H;
        }
        if zero {
            flags |= FLAG_Z;
        }
        self.write_reg(Reg::F, flags);

        res
    }

    /// Sumar dos valores de 8-bits + el carry si el flag estaba activado de
    /// alguna operción anterior
    // NOTE: Esto produce un ADC en x64? espero, sino emos sido engañados
    #[inline]
    fn alu_adc(&mut self, a: u8, b: u8) -> u8 {
        // Realizar la operación y decidir que flags se activan
        let (mut res, mut carry) = a.overflowing_add(b);
        let half_carry = res >> 4 != 0;
        let zero = res == 0;

        // Sumar el carry si la flag está activada
        let mut flags = self.read_reg(Reg::F); 
        if flags & FLAG_C != 0 {
            let (new_res, new_carry) = res.overflowing_add(1);
            res = new_res;
            carry |= new_carry;
        }

        // Crear el u8 de flags de la operación
        flags = 0;
        if carry {
            flags |= FLAG_C;
        }
        if half_carry {
            flags |= FLAG_H;
        }
        if zero {
            flags |= FLAG_Z;
        }
        self.write_reg(Reg::F, flags);

        res
    }

    /// Restar dos valores de 8-bits de la alu
    #[inline]
    fn alu_sub(&mut self, a: u8, b: u8) -> u8 {
        // Realizar la operación y decidir que flags se activan
        let (res, carry) = a.overflowing_sub(b);

        // FIXME: No se si esto realmente calcula el borrow a partir de 4-bit
        // tal como dice la spec, simplemente me pareció la solución naive
        let half_carry = b >> 4 != 0;
        let zero = res == 0;

        // Crear el u8 de flags de la operación
        let mut flags = FLAG_N;
        if !carry {
            flags |= FLAG_C;
        }
        if !half_carry {
            flags |= FLAG_H;
        }
        if zero {
            flags |= FLAG_Z;
        }
        self.write_reg(Reg::F, flags);

        res
    }

    /// Restar dos valores de 8-bits - el carry si el flag estaba activado de
    /// alguna operción anterior
    #[inline]
    fn alu_sbc(&mut self, a: u8, b: u8) -> u8 {
        // Realizar la operación y decidir que flags se activan
        let (mut res, mut carry) = a.overflowing_sub(b);

        // FIXME: No se si esto realmente calcula el borrow a partir de 4-bit
        // tal como dice la spec, simplemente me pareció la solución naive
        let half_carry = b >> 4 != 0;
        let zero = res == 0;

        // Sumar el carry si la flag está activada
        // FIXME: Según la spec es sumar el carry a la solución
        let mut flags = self.read_reg(Reg::F); 
        if flags & FLAG_C != 0 {
            let (new_res, new_carry) = res.overflowing_add(1);
            res = new_res;
            carry |= new_carry;
        }

        // Crear el u8 de flags de la operación
        flags = FLAG_N;
        if !carry {
            flags |= FLAG_C;
        }
        if !half_carry {
            flags |= FLAG_H;
        }
        if zero {
            flags |= FLAG_Z;
        }
        self.write_reg(Reg::F, flags);

        res
    }

    #[inline]
    fn alu_and(&mut self, a: u8, b: u8) -> u8 {
        // Relizar la operación y decidir que flags se activan
        let res = a & b;
        let zero = res == 0;

        // Crear el u8 de flags de la operación
        let mut flags = FLAG_H;
        if zero {
            flags |= FLAG_Z;
        }
        self.write_reg(Reg::F, flags);

        res
    }

    #[inline]
    fn alu_or(&mut self, a: u8, b: u8) -> u8 {
        // Relizar la operación y decidir que flags se activan
        let res = a | b;
        let zero = res == 0;

        // Crear el u8 de flags de la operación
        let mut flags = 0;
        if zero {
            flags |= FLAG_Z;
        }
        self.write_reg(Reg::F, flags);

        res
    }

    #[inline]
    fn alu_rlc(&mut self, a: u8) -> u8 {
        // Hacer la operación rotate por 1 a izquierda
        let res = a.rotate_left(1);

        // Extraer y aplicar los flags
        let carry = a >> 7 == 1;
        let zero = res == 0;
        let mut flags = 0;
        if carry {
            flags |= FLAG_C;
        }
        if zero {
            flags |= FLAG_Z;
        }
        self.write_reg(Reg::F, flags);

        res
    }

    #[inline]
    fn alu_rrc(&mut self, a: u8) -> u8 {
        // Hacer la operación rotate por 1 a derecha
        let res = a.rotate_right(1);

        // Extraer y aplicar los flags
        let carry = a & 0b11111110 == 1;
        let zero = res == 0;
        let mut flags = 0;
        if carry {
            flags |= FLAG_C;
        }
        if zero {
            flags |= FLAG_Z;
        }
        self.write_reg(Reg::F, flags);

        res
    }

    #[inline]
    fn alu_rl(&mut self, a: u8) -> u8 {
        // Extraer la carry flag
        let carry = (self.read_reg(Reg::F) & FLAG_C != 0) as u8;

        // Hacer la operación rotate por 1 a izquierda
        let res = a.rotate_left(carry as u32);

        // Extraer y aplicar los flags
        let carry = a >> 7 == 1;
        let zero = res == 0;
        let mut flags = 0;
        if carry {
            flags |= FLAG_C;
        }
        if zero {
            flags |= FLAG_Z;
        }
        self.write_reg(Reg::F, flags);

        res
    }

    #[inline]
    fn alu_rr(&mut self, a: u8) -> u8 {
        // Extraer la carry flag
        let carry = (self.read_reg(Reg::F) & FLAG_C != 0) as u8 ;

        // Hacer la operación rotate por 1 a izquierda
        let res = a.rotate_right(carry as u32);

        // Extraer y aplicar los flags
        let carry = a & 0b11111110 == 1;
        let zero = res == 0;
        let mut flags = 0;
        if carry {
            flags |= FLAG_C;
        }
        if zero {
            flags |= FLAG_Z;
        }
        self.write_reg(Reg::F, flags);

        res
    }

    #[inline]
    fn alu_sla(&mut self, a: u8) -> u8 {
        // Hacer la operación shift por 1 a izquierda
        let (res, carry) = a.overflowing_shl(1);

        // Extraer y aplicar los flags
        let zero = res == 0;
        let mut flags = 0;
        if carry {
            flags |= FLAG_C;
        }
        if zero {
            flags |= FLAG_Z;
        }
        self.write_reg(Reg::F, flags);

        res
    }

    #[inline]
    fn alu_sra(&mut self, a: u8) -> u8 {
        // Hacer la operación shift por 1 a derecha
        let (mut res, carry) = a.overflowing_shr(1);
        res |= a & 0b10000000;

        // Extraer y aplicar los flags
        let zero = res == 0;
        let mut flags = 0;
        if carry {
            flags |= FLAG_C;
        }
        if zero {
            flags |= FLAG_Z;
        }
        self.write_reg(Reg::F, flags);

        res
    }

    #[inline]
    fn alu_swap(&mut self, a: u8) -> u8 {
        // Intercambiar los nimbles
        let hnimble = a >> 4;
        let lnimble = a & 0b11110000;
        let res = (lnimble << 4) | hnimble;
     
        // Extraer y aplicar los flags
        let zero = res == 0;
        let mut flags = 0;
        if zero {
            flags |= FLAG_Z;
        }
        self.write_reg(Reg::F, flags);

        res
    }

    #[inline]
    fn alu_srl(&mut self, a: u8) -> u8 {
        // Hacer la operación shift por 1 a derecha
        let (res, carry) = a.overflowing_shr(1);

        // Extraer y aplicar los flags
        let zero = res == 0;
        let mut flags = 0;
        if carry {
            flags |= FLAG_C;
        }
        if zero {
            flags |= FLAG_Z;
        }
        self.write_reg(Reg::F, flags);

        res
    }

    #[inline]
    fn alu_bit(&mut self, a: u8, bit: u8) {
        // Comprobar si el bit `bit` es cero
        let is_zero = a & (1 << bit) == 0;

        // Aplicar los flags necesarios
        let old_carry = self.read_reg(Reg::F);
        let mut flags = old_carry | FLAG_H;
        if is_zero {
            flags |= FLAG_Z;
        }
        self.write_reg(Reg::F, flags);
    }

    #[inline]
    fn alu_res(&mut self, a: u8, bit: u8) -> u8 {
        a & !(1 << bit)
    }

    #[inline]
    fn alu_set(&mut self, a: u8, bit: u8) -> u8 {
        a | (1 << bit)
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
                let res = self.alu_add(self.read_reg(src), self.read_reg(dst));
                self.write_reg(dst, res);
            },
            Instr::AddRegImm { src, dst } => {
                tick!(self, 8);
                let res = self.alu_add(src, self.read_reg(dst));
                self.write_reg(dst, res);
            },
            Instr::AddMemReg { .. } => todo!(),
            Instr::AddWRegWReg { src, dst } => {
                tick!(self, 8);
                let res = self.alu_wideadd(self.read_widereg(src), 
                    self.read_widereg(dst));
                self.write_widereg(dst, res);

            },
            Instr::AddWRegImm { src, dst } => {
                tick!(self, 16);
                let res = self.alu_wideadd(src as u16, self.read_widereg(dst));
                self.write_widereg(dst, res);
            }
            Instr::AdcRegReg { src, dst } => {
                tick!(self, 4);
                let res = self.alu_adc(self.read_reg(src), self.read_reg(dst));
                self.write_reg(dst, res);
            },
            Instr::AdcRegImm { src, dst } => {
                tick!(self, 8);
                let res = self.alu_adc(src, self.read_reg(dst));
                self.write_reg(dst, res);
            },
            Instr::AdcMemReg { .. } => todo!(),
            Instr::SubReg { src } => {
                tick!(self, 4);
                let res = self.alu_sub(self.read_reg(Reg::A), self.read_reg(src));
                self.write_reg(Reg::A, res);
            },
            Instr::SubImm { src } => {
                tick!(self, 8);
                let res = self.alu_sub(self.read_reg(Reg::A), src);
                self.write_reg(Reg::A, res);
            },
            Instr::SubMem { .. } => todo!(),
            Instr::SbcReg { src } => {
                tick!(self, 4);
                let res = self.alu_sbc(self.read_reg(Reg::A), self.read_reg(src));
                self.write_reg(Reg::A, res);
            },
            Instr::SbcImm { src } => {
                tick!(self, 8);
                let res = self.alu_sbc(self.read_reg(Reg::A), src);
                self.write_reg(Reg::A, res);
            },
            Instr::SbcMem { .. } => todo!(),
            Instr::AndReg { src } => {
                tick!(self, 4);
                let res = self.alu_and(self.read_reg(Reg::A), self.read_reg(src));
                self.write_reg(Reg::A, res);
            },
            Instr::AndImm { src } => {
                tick!(self, 8);
                let res = self.alu_and(self.read_reg(Reg::A), src);
                self.write_reg(Reg::A, res);
            },
            Instr::AndMem { .. } => todo!(),
            Instr::OrReg { src } => {
                tick!(self, 4);
                let res = self.alu_or(self.read_reg(Reg::A), self.read_reg(src));
                self.write_reg(Reg::A, res);
            },
            Instr::OrImm { src } => {
                tick!(self, 8);
                let res = self.alu_or(self.read_reg(Reg::A), src);
                self.write_reg(Reg::A, res);
            },
            Instr::OrMem { .. } => todo!(),
            Instr::IncReg { dst } => {
                tick!(self, 4);
                let res = self.alu_add(self.read_reg(dst), 1);
                self.write_reg(dst, res);

                // Los incrementos no modifican el flag de carry
                let flags = self.read_reg(Reg::F) ^ FLAG_C;
                self.write_reg(Reg::F, flags);
            },
            Instr::IncWReg { dst } => {
                tick!(self, 8);
                let res = self.alu_wideadd(self.read_widereg(dst), 1);
                self.write_widereg(dst, res);

                // Los incrementos no modifican el flag de carry
                let flags = self.read_reg(Reg::F) ^ FLAG_C;
                self.write_reg(Reg::F, flags);
            },
            Instr::IncMem { .. } => todo!(),
            Instr::DecReg { dst } => {
                tick!(self, 4);
                let res = self.alu_sub(self.read_reg(dst), 1);
                self.write_reg(dst, res);

                // Los incrementos no modifican el flag de carry
                let flags = self.read_reg(Reg::F) ^ FLAG_C;
                self.write_reg(Reg::F, flags);
            },
            Instr::DecWReg { dst } => {
                tick!(self, 8);
                let res = self.read_widereg(dst).checked_sub(1).unwrap_or(0);
                self.write_widereg(dst, res);

                // Los decrementos no modifican los flags
            },
            Instr::DecMem { .. } => todo!(),
            Instr::CpReg { src } => {
                tick!(self, 4);
                self.alu_sub(self.read_reg(Reg::A), self.read_reg(src));
            },
            Instr::CpImm { src } => {
                tick!(self, 8);
                self.alu_sub(self.read_reg(Reg::A), src);
            },
            Instr::CpMem { .. } => todo!(),
            Instr::LdWRegImm { src, dst } => {
                tick!(self, 12);
                self.write_widereg(dst, src);
            },
            Instr::LdMemImmReg { .. } => todo!(),
            Instr::Push { src } => {
                tick!(self, 16);
                let _value = self.read_widereg(src);

                todo!();
            },
            Instr::Pop { .. } => todo!(),
            Instr::JPImm { addr } => {
                tick!(self, 16);
                self.pc = addr;
            },
            Instr::JPCond { cond, addr } => {
                tick!(self, 12);

                // Comprobar que almenos todos los bits de la condición están
                // a 1
                let flags = self.read_reg(Reg::F);
                if flags & cond != cond {
                    return Some(());
                }

                tick!(self, 4);

                self.pc = addr;
            },
            Instr::JPReg { .. } => todo!(),
            Instr::JRelImm { offset } => {
                tick!(self, 8);

                // Añadir el offset a pc
                let offset = offset as i16;
                self.pc = (self.pc as i16 + offset).try_into()
                    .expect("After a relative jump `pc` is negative");
            },
            Instr::JRelCond { cond, offset } => {
                tick!(self, 8);

                // Comprobar que almenos todos los bits de la condición están
                // a 1
                let flags = self.read_reg(Reg::F);
                if flags & cond != cond {
                    return Some(());
                }
                
                tick!(self, 4);

                // Añadir el offset a pc
                let offset = offset as i16;
                self.pc = (self.pc as i16 + offset).try_into()
                    .expect("After a relative jump `pc` is negative");
            },
            Instr::Rst { addr } => {
                tick!(self, 8);

                // Mover la dirección actual al stack
                let [curr_addr_h, curr_addr_l] = self.pc.to_le_bytes();
                todo!();
            },
            Instr::RlcReg { reg } => {
                tick!(self, 8);
                let res = self.alu_rlc(self.read_reg(reg));
                self.write_reg(reg, res);
            },
            Instr::RlcMem { .. } => todo!(),
            Instr::RrcReg { reg } => {
                tick!(self, 8);
                let res = self.alu_rrc(self.read_reg(reg));
                self.write_reg(reg, res);
            },
            Instr::RrcMem { .. } => todo!(),
            Instr::RlReg { reg } => {
                tick!(self, 8);
                let res = self.alu_rl(self.read_reg(reg));
                self.write_reg(reg, res);
            },
            Instr::RlMem { .. } => todo!(),
            Instr::RrReg { reg } => {
                tick!(self, 8);
                let res = self.alu_rr(self.read_reg(reg));
                self.write_reg(reg, res);
            },
            Instr::RrMem { .. } => todo!(),
            Instr::SlaReg { reg } => {
                tick!(self, 8);
                let res = self.alu_sla(self.read_reg(reg));
                self.write_reg(reg, res);
            },
            Instr::SlaMem { .. } => todo!(),
            Instr::SraReg { reg } => {
                tick!(self, 8);
                let res = self.alu_sra(self.read_reg(reg));
                self.write_reg(reg, res);
            },
            Instr::SwapReg { reg } => {
                tick!(self, 8);
                let res = self.alu_swap(self.read_reg(reg));
                self.write_reg(reg, res);
            },
            Instr::SwapMem { .. } => todo!(),
            Instr::SrlReg { reg } => {
                tick!(self, 8);
                let res = self.alu_srl(self.read_reg(reg));
                self.write_reg(reg, res);
            },
            Instr::BitReg { reg, bit } => {
                tick!(self, 8);
                self.alu_bit(self.read_reg(reg), bit);
            },
            Instr::BitMem { .. } => todo!(),
            Instr::ResReg { reg, bit } => {
                tick!(self, 8);
                self.alu_res(self.read_reg(reg), bit);
            },
            Instr::ResMem { .. } => todo!(),
            Instr::SetReg { reg, bit } => {
                tick!(self, 8);
                self.alu_set(self.read_reg(reg), bit);
            },
            Instr::SetMem { .. } => todo!(), 
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
