use std::{ffi::{c_void, CStr}, str::Utf8Error};

use windows::Win32::{Foundation::HANDLE, System::Threading::PROCESS_ALL_ACCESS};

pub struct Memory {
    bytes: Vec<u8>,
    virtual_address: usize,
    pid: u32,
    handle: HANDLE
}

impl Memory {
    pub fn new(virtual_address: usize, capacity: usize, pid: u32) -> Memory {
        let handle = unsafe { windows::Win32::System::Threading::OpenProcess(PROCESS_ALL_ACCESS, false, pid)
            .expect("Could not open process.") };

        Memory {
            bytes: vec![0; capacity],
            pid: pid,
            virtual_address: virtual_address,
            handle: handle
        }
    }

    pub fn refresh(&mut self) {
        unsafe {
            let _ = windows::Win32::System::Diagnostics::Debug::ReadProcessMemory(
                self.handle, 
                self.virtual_address as *const c_void, 
                self.bytes.as_mut_ptr() as *mut c_void,
                self.bytes.len(), 
                None
            );
        }
    }

    pub fn read<T>(&self, physical_address: u32) -> T {
        let physical_address = Memory::fix_pointer(physical_address);
        unsafe { std::ptr::read(self.bytes[physical_address as usize..].as_ptr() as *const _) }
    }

    pub fn read_str(&self, physical_address: u32) -> Result<&str, Utf8Error> {
        let physical_address = Memory::fix_pointer(physical_address);
        unsafe { CStr::from_ptr(self.bytes[physical_address as usize..].as_ptr() as *const _).to_str() }
    }

    pub fn write(&mut self, physical_address: u32, write_bytes: &[u8]) {
        let write_address = physical_address as usize + self.virtual_address;
        unsafe {
            let res = windows::Win32::System::Diagnostics::Debug::WriteProcessMemory(
                self.handle, 
                write_address as *const c_void, 
                write_bytes.as_ptr() as *mut c_void, 
                write_bytes.len(), 
                None
            );

            if let Err(e) = res {
                println!("Res: {}", e);
            }
        }
    }

    pub fn fix_pointer(ptr: u32) -> u32 {
        let mut ptr = ptr.to_le_bytes();
        ptr[3] = 0x0;
        u32::from_le_bytes(ptr)
    }
}

impl Drop for Memory {
    fn drop(&mut self) {
        if !self.handle.is_invalid() {
            unsafe {
                let _ = windows::Win32::Foundation::CloseHandle(self.handle);
            }
        }
    }
}