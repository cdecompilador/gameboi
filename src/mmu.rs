use std::ops::Range;

/// Variantes que controlan el acceso de lectura a memoria desde CPU
pub enum MemRead {
    /// Se reemplaza el valor que quiere leer la CPU por otro
    Replace(u8),

    /// Muestra el valor que hay realmente en memoria a la CPU
    PassThrough,
}

/// Variantes que controlan el acceso de escritura a memoria desde CPU
pub enum MemWrite {
    /// Se reemplaza el valor que quiere escribir la CPU por otro
    Replace(u8),

    /// Permite la escritura
    PassThrough,

    /// No permite la escritura y falla silencionamente
    Block,
}

pub struct MemHandler {
    /// La función es llamada cuando al CPU intenta leer desde memoria y hay
    /// un handler a esa región
    on_read: fn(mmu: &Mmu, addr: Addr) -> MemRead,

    /// La función es llamada cuando al CPU intenta escribir a memoria y hay
    /// un handler a esa región
    on_write: fn(mmu: &Mmu, addr: Addr, value: u8) -> MemWrite,
}

const IO_HANDLE: MemHandler = MemHandler {
    on_read: |mmu: &Mmu, addr: Addr| -> MemRead {
        MemRead::PassThrough
    },
    on_write: |mmu: &Mmu, addr: Addr, value: u8| -> MemWrite {
        MemWrite::PassThrough
    },
};

pub struct Addr(u16);

impl Addr {
    pub fn get_handler(mmu: &Mmu) -> MemHandler {
        todo!()
    }
}

/*
struct MemHandlers {
    mem_handlers_ranges: Vec<Range<usize>>,
    mem_handlers: Vec<MemHandler>,
}

impl MemHandlers {
    fn new() -> Self {
        Self {
            mem_handler_ranges: Vec::new(),
            mem_handlers: Vec::new()
        }
    }
}
*/

pub struct Mmu {
    memory: [u8; u16::MAX as usize],
}

impl Mmu {
    pub fn new() -> Self {
        Self {
            memory: [0; u16::MAX as usize],
        }
    }

    pub fn read_word(&self, addr: Addr) -> Option<u8> {
        self.memory.get(addr.0 as usize)
    }

    pub fn write_word(&mut self, addr: Addr, value: u8) -> Option<()> c{
        *self.memory.get_mut(addr.0 as usize) = value;
    }

    pub fn read_dword(&self, addr: Addr) -> Option<u16> {
        let h = self.memory.get(addr.0 as usize)?;
        let l = self.memory.get(addr.0.checked_add(1)? as usize)?;
        Some(u16::from_le_bytes([h, l]))
    }

    pub fn write_dword(&mut self, addr: Addr, value: u16) {

    }
}
