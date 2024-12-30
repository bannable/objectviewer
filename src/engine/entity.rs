use std::marker::PhantomData;

use crate::memory::Memory;

const AT_T_AT_D: u32 = 1681945664;

#[derive(Debug)]
#[repr(C)]
pub struct EntityManager<T> {
    pub name: [u8; 32],
    pub max_entries: u16,
    pub data_sizeof: u16,
    pub valid: u8, // What is this used for?
    pub identifier_zero_invalid: u8,
    pub unknown_1: u16,
    pub signature: u32,
    pub next_index: u16,
    pub capacity: u16,
    pub size: u16,
    pub next_id: u16,
    pub data_begin: u32,
    _phantom: PhantomData<T>
}

impl<T> EntityManager<T> {
    fn is_signature_valid(&self) -> bool {
        self.signature == AT_T_AT_D
    }

    pub fn is_valid(&self) -> bool {
        self.is_signature_valid() && self.valid == 1
    }

    pub fn read(&self, memory: &Memory) -> Vec<Option<T>> {
        let mut entries: Vec<_> = (0..self.max_entries).map(|_| None).collect();

        let data_begin = Memory::fix_pointer(self.data_begin);
        for index in (0..self.capacity as usize).rev() {
            let index_ptr_addr = data_begin + (self.data_sizeof as u32 * index as u32);
            let id: u16 = memory.read(index_ptr_addr);

            if id != 0 || self.identifier_zero_invalid == 0 { 
                let entry: T = memory.read(index_ptr_addr);
                entries[index] = Some(entry);
            }
        }

        entries
    }
}
