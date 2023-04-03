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
    Imm,
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
    AddRegMem = 9,
    AddWRegWReg = 10,
    AddWRegImm = 11,

    AdcRegReg = 12,
    AdcRegImm = 13,
    AdcRegMem = 14,

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
    Push = 41,
    Pop = 42,
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
    AddWRegImm { src: u8, dst: Reg },

    AdcRegReg { src: Reg, dst: Reg },
    AdcRegImm { src: u8, dst: Reg },
    AdcRegMem { src: RegAddr, dst: Reg },

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
    Push { src: Reg },
    Pop { dst: Reg },
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
    0, 0, 4, 0, 0, 0, 3, 0, 0,10, 5, 0, 0, 0, 3, 0,
    0, 0, 4, 0, 0, 0, 3, 0, 0,10, 5, 0, 0, 0, 3, 0,
    0, 0, 4, 0, 0, 0, 3, 0, 0,10, 5, 0, 0, 0, 3, 0,
    0, 0, 4, 0, 0, 0, 3, 0, 0,10, 5, 0, 0, 0, 3, 0,
    2, 2, 2, 2, 2, 2, 5, 2, 2, 2, 2, 2, 2, 2, 5, 2,
    2, 2, 2, 2, 2, 2, 5, 2, 2, 2, 2, 2, 2, 2, 5, 2,
    2, 2, 2, 2, 2, 2, 5, 2, 2, 2, 2, 2, 2, 2, 5, 2,
    4, 4, 4, 4, 4, 4, 1, 4, 2, 2, 2, 2, 2, 2, 5, 2,
    7, 7, 7, 7, 7, 7, 9, 7,12,12,12,12,12,12,14,12,
   15,15,15,15,15,15,17,15,18,18,18,18,18,18,18,18,
   21,21,21,21,21,21,23,21,24,24,24,24,24,24,24,24,
   27,27,27,27,27,27,27,28, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 2, 0, 0, 0, 9, 0, 0, 0, 0, 0, 0, 0,13, 0,
    0, 0, 4, 0, 0, 0,16, 0, 0, 0, 0, 0, 0, 0,19, 0,
    0, 0, 0, 0, 0, 0,22, 0,11, 0, 0, 0, 0, 0,25, 0,
    0, 0, 0, 0, 0, 0,28, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

/// Tabla usada para discernir el operando de entrada de la instrucción, sus
/// valores son convertibles directamente a los enums `Reg` y `RegMem`, en 
/// release la conversión se hace sin comprobaciones
const SRC_TABLE: &[u8] = &[
    0, 0, 1, 3, 3, 3, 0, 0, 9, 3,13, 3, 4, 3, 0, 0,
    0, 0, 1, 5, 5, 5, 0, 0, 0, 5,14, 5, 6, 5, 0, 0,
    0, 0, 1, 7, 7, 7, 0, 0, 0, 7,11, 7, 8, 8, 0, 0,
    0, 0, 1, 9,10,10, 0, 0, 0, 9,12, 9, 1, 1, 0, 0,
    3, 3, 4, 6, 7, 8,10, 1, 3, 4, 5, 6, 7, 8,10, 1,
    3, 4, 5, 6, 7, 8,10, 1, 3, 4, 5, 6, 7, 8,10, 1,
    3, 4, 5, 6, 7, 8,10, 1, 3, 4, 5, 6, 7, 8,10, 1,
    3, 4, 5, 6, 7, 8, 0, 1, 3, 4, 5, 6, 7, 8,10, 1,
    3, 4, 5, 6, 7, 8,10, 1, 3, 4, 5, 6, 7, 8,10, 1,
    3, 4, 5, 6, 7, 8,10, 1, 3, 4, 5, 6, 7, 8,10, 1,
    3, 4, 5, 6, 7, 8,10, 1, 3, 4, 5, 6, 7, 8,10, 1,
    3, 4, 5, 6, 7, 8,10, 1, 3, 4, 5, 6, 7, 8,10, 1,
    0, 3, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 5, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 7, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
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

                Some(Instr::LdRegImm { src: imm, dst })        
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

                Some(Instr::LdRegMem { src, dst })
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

                Some(Instr::LdMemReg { src, dst })
            }}
        }

        match InstrKind::from_u8(INST_KIND_TABLE[opcode as usize]) {
            InstrKind::Halt => Some(Instr::Halt),
            InstrKind::LdRegReg => decode_reg_reg!(LdRegReg),
            InstrKind::LdRegImm => decode_reg_imm!(LdRegImm),
            InstrKind::LdRegMem => decode_reg_mem!(LdRegMem),
            InstrKind::LdMemReg => decode_mem_reg!(LdMemReg),
            InstrKind::AddRegReg => decode_reg_reg!(AddRegReg),
            InstrKind::AddRegImm => decode_reg_imm!(AddRegImm),
            InstrKind::AddRegMem => decode_reg_mem!(AddRegMem),
            InstrKind::AddWRegWReg => decode_reg_reg!(AddWRegWReg),
            InstrKind::AdcRegReg => decode_reg_reg!(AdcRegReg),
            InstrKind::AdcRegImm => decode_reg_imm!(AdcRegImm),
            InstrKind::AdcRegMem => decode_reg_mem!(AdcRegMem),
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
                let imm = u16::from_ne_bytes([immh, imml]);
                
                // Extraer registro destino
                let dst = DST_TABLE[opcode as usize];

                assert!(dst != 0);

                let dst = Reg::from_u8(dst);

                Some(Instr::LdWRegImm { src: imm, dst })
            },
            InstrKind::Push => decode_reg!(src, Push),
            InstrKind::Pop  => decode_reg!(dst, Pop),
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
            Instr::AddRegMem { .. } => todo!(),
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
            Instr::AdcRegMem { .. } => todo!(),
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
            Instr::Push { src } => {
                let value = self.read_widereg(src);

                todo!();
            },
            Instr::Pop { .. } => todo!(),
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
