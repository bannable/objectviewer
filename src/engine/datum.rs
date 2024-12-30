use std::fmt;

const INVALID_HANDLE: u32 = 4294967295;

#[repr(C)]
#[derive(Clone)]
pub struct Datum(u32);

impl Datum {
    pub fn from_parts(index: u16, id: u16) -> Datum {
        Datum(((id as u32) << 16) | index as u32) 
    }

    pub fn from_raw(handle: u32) -> Datum {
        Datum(handle) 
    }

    pub fn get_index(&self) -> u16 {
        let lower_word = self.0 & 0xFFFF;
        lower_word as u16
    }

    pub fn get_id(&self) -> u16 {
        let upper_word = (self.0 >> 16) & 0xFFFF;
        upper_word as u16
    }

    pub fn get_handle(&self) -> u32 {
        self.0
    }

    pub fn is_invalid(&self) -> bool {
        self.get_handle() == INVALID_HANDLE
    }
}

impl fmt::Debug for Datum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Datum")
         .field("Handle", &self.get_handle())
         .field("Index", &self.get_index())
         .field("ID", &self.get_id())
         .finish()
    }
}