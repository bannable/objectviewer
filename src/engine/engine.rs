use std::collections::HashMap;

use crate::memory::Memory;

use super::{datum::Datum, EntityManager};

// Halo 1 Xbox Retail
const HALO_OBJECT_POOL_HEADER_ADDR: u32 = 0x000B9370;
const HALO_PLAYER_POOL_HEADER_ADDR: u32 = 0x00213C50;

const HALO_TAG_HEADER_ADDR: u32 = 0x003A6000; 
const HALO_PLAYER_GLOBALS_ADDR: u32 = 0x00214E00;

// Sanity check constants
const DEAH: u32 = 1751474532;
const LIAT: u32 = 1952541036;
const RNCS: u32 = 1935896178;

// Halo Structs
const MAXIMUM_NUMBER_OF_LOCAL_PLAYERS: usize = 4;

#[derive(Debug)]
#[repr(C)]
pub struct PlayersGlobals {
    pub unknown_1: i32,
    pub local_players: [Datum; MAXIMUM_NUMBER_OF_LOCAL_PLAYERS],
    pub local_dead_players: [Datum; MAXIMUM_NUMBER_OF_LOCAL_PLAYERS],
    pub local_player_count: u16,
    pub double_speed_ticks_remaining: u16,
    pub are_all_dead: u8,
    pub input_disabled: u8,
    pub unk_tag_index: u16, // bsp index??
    pub respawn_failure: u16,
    pub teleported: u8,
    pub unk_flags: u8,
    pub combined_pvs: [u8; 0x40],
    pub combined_pvs_local: [u8; 0x40]
}

#[derive(Debug)]
#[repr(C)]
pub struct PlayerDataEntry {
    pub id: u16,
    pub local_player_index: u16,
    pub player_name: [u16; 12],
    pub unknown_1: [i32; 6],
    pub slave_unit_index: Datum, // datum
    pub last_slave_unit_index: Datum, // datum
    pub unknown_2: [u8; 150]
}

#[derive(Debug)]
#[repr(C)]
pub struct TagEntry {
    pub tag_class: u32,
    pub tag_class_secondary: u32,
    pub tag_class_tertiary: u32,
    pub tag_index: u32,
    tag_path_ptr: u32,
    tag_data_ptr: u32,
    unknown_1: u32,
    unknown_2: u32
}

#[derive(Debug)]
#[repr(C)]
pub struct TagHeader {
    tag_array_ptr: u32,
    tag_index: u32,
    map_id: u32,
    tag_count: u32,
    vertex_count: u32,
    vertex_offset: u32,
    index_count: u32,
    index_offset: u32,
    model_data_size: u32,
    footer: u32 // tags backwards
}

#[derive(Debug)]
#[repr(C)]
pub struct ObjectHeaderEntry {
    pub id: u16,
    pub unk_2: u8,
    pub data_type: u8,
    pub unknown_2: u16,
    pub data_sizeof: u16,
    pub object_address: u32
}

#[derive(Debug)]
#[repr(C)]
pub struct ObjectListHeader {
    pub header_head: u32,
    pub tag_id: u32, 
    pub ptr_a: u32,
    pub ptr_next_object: u32,
    pub ptr_previous_object: u32,
    pub header_tail: u32,
}

const NUMBER_OF_OUTGOING_OBJECT_FUNCTIONS: usize = 4;
const MAXIMUM_REGIONS_PER_OBJECT: usize = 8;

#[derive(Debug)]
#[repr(C)]
pub struct Object {
    pub tag_index: u32,
    pub flags: u32,
    pub unk_8: u32,
    pub position: [f32; 3],
    pub unk_24: [f32; 3],
    pub unk_36: [f32; 3],
    pub unk_48: [f32; 3],
    pub unk_60: [f32; 3],
    pub unk_72: u32,
    pub unk_76: Datum,
    pub unk_80: f32,
    pub unk_84: f32,
    pub unk_88: f32,
    pub unk_92: f32,
    pub unk_96: f32,
    pub object_type: i16,
    pub unk_102: i16,
    pub unk_104: i16,
    pub unk_106: i16,
    pub unk_108: i16,
    pub unk_110: i16,
    pub unk_112: u32,
    pub unk_116: u32,
    pub unk_120: u32,
    pub unk_124: u32,
    pub unk_128: i16,
    pub unk_130: i16,
    pub unk_132: i16,
    pub unk_134: i16,
    pub unk_136: u32,
    pub unk_140: f32,
    pub unk_144: f32,
    pub unk_148: f32,
    pub unk_152: u32,
    pub unk_156: f32,
    pub unk_160: u32,
    pub unk_164: f32,
    pub unk_168: f32,
    pub unk_172: u32,
    pub unk_176: u32,
    pub unk_180: i16,
    pub unk_182: i8,
    pub unk_183: i8,
    pub unk_184: u32,
    pub unk_188: u32,
    pub unk_192: u32,
    pub next_object_index: Datum,
    pub unk_200: Datum,
    pub parent_object_index: Datum,
    pub unk_208: [f32; 5],
    pub unk_228: [f32; NUMBER_OF_OUTGOING_OBJECT_FUNCTIONS],
    pub unk_244: [u8; 8],
    pub unk_252: [u8; 32],
    pub unk_284: u32,
    pub unk_288: u32,
    pub unk_292: u16,
    pub unk_294: u16,
    pub unk_296: [u8; MAXIMUM_REGIONS_PER_OBJECT],
    pub unk_304: [u8; MAXIMUM_REGIONS_PER_OBJECT],
    pub unk_312: [u8; 0x60],
    pub unk_408: u32,
    pub unk_412: u32,
    pub unk_416: u32,
}

pub fn object_type_string(data_type: u8) -> &'static str {
    match data_type {
        0 => "bipd", // unit obje
        1 => "vehi", // unit obje
        2 => "weap", // item obje
        3 => "eqip", // item obje
        4 => "garb", // item obje
        5 => "proj", // obje
        6 => "scen", // obje
        7 => "mach", // devi obje
        8 => "ctrl", // devi obje
        9 => "lifi", // devi obje
        11 => "ssce", // obje
        _ => "UNKNOWN"
    }
}

// Application
#[derive(Debug)]
pub struct EngineSnapshot {
    pub player_header: EntityManager<PlayerDataEntry>,
    pub player_entries: Vec<Option<PlayerDataEntry>>,
    pub object_header: EntityManager<ObjectHeaderEntry>,
    pub object_header_entries: Vec<Option<ObjectHeaderEntry>>, 
    pub object_entries: Vec<Option<Object>>,
    pub player_globals: PlayersGlobals,
    pub tags: HashMap<u32, String>,
    pub tag_entries: HashMap<u32, TagEntry>
}

impl EngineSnapshot {
    // We only check the index here as the game does not safely check the ID
    // Otherwise, during some AUP we would not see the proper assignments for players.
    pub fn find_local_player_index_from_unit_index(&self, unit_index: u16) -> Option<u16> {
        for player_pool_entry in self.player_entries.iter() {

            if player_pool_entry.is_none() { continue; }
            let player_pool_entry = player_pool_entry.as_ref().unwrap();

            if player_pool_entry.slave_unit_index.get_index() == unit_index {
                return Some(player_pool_entry.local_player_index);
            }
        }

        None
    }

    pub fn find_next_object_datum_player(&self, object_handle: Datum) -> Option<u16> {
        for player_object_handle in self.player_globals.local_dead_players.iter() {
            let player_index = player_object_handle.get_index();
            let object_index = object_handle.get_index();

            if player_index == object_index {
                return Some(player_index);
            }
        }
            
        None
    }

}

pub fn build_snapshot(memory: &Memory) -> Option<EngineSnapshot> {
    let tag_header: TagHeader = memory.read(HALO_TAG_HEADER_ADDR);
    if tag_header.footer != RNCS { return None; }

    let object_manager: EntityManager<ObjectHeaderEntry> = memory.read(HALO_OBJECT_POOL_HEADER_ADDR);
    if !object_manager.is_valid() { return None; }

    let player_manager: EntityManager<PlayerDataEntry> = memory.read(HALO_PLAYER_POOL_HEADER_ADDR);
    if !player_manager.is_valid() { return None; }

    let object_pool_entries = object_manager.read(memory);
    let player_pool_entries = player_manager.read(memory);

    let mut game_object_entries: Vec<_> = (0..object_pool_entries.len()).map(|_| None).collect(); // FIX ME
    for index in 0..object_manager.capacity as usize {
        let object_entry = &object_pool_entries[index];
        if object_entry.is_none() { continue; }
        
        let object_entry = object_entry.as_ref().unwrap();
        let object_address = Memory::fix_pointer(object_entry.object_address);
        if object_address != 0 && object_address >= size_of::<ObjectListHeader>() as u32 {
            let object_list_header_ptr = object_address - size_of::<ObjectListHeader>() as u32;
            let object_list_header: ObjectListHeader = memory.read(object_list_header_ptr);

            if !(object_list_header.header_head == DEAH && object_list_header.header_tail == LIAT) {
                return None;
            }

            let game_object: Object = memory.read(object_address);
            game_object_entries[index] = Some(game_object);
        }
    }

    // TODO: Find a way to sanity check this data
    let player_globals: PlayersGlobals = memory.read(HALO_PLAYER_GLOBALS_ADDR);

    // Get tag index mappings to tag names
    // Also store the tag entries
    let mut tag_index_to_tag_entry: HashMap<u32, TagEntry> = HashMap::new();
    let mut tag_index_to_str: HashMap<u32, String> = HashMap::new();

    let tag_array_base_ptr = Memory::fix_pointer(tag_header.tag_array_ptr);
    for index in 0..tag_header.tag_count {
        let tag_entry_ptr = tag_array_base_ptr + (size_of::<TagEntry>() as u32 * index);
        let tag_entry: TagEntry = memory.read(tag_entry_ptr);

        // Also used for tag_index_to_tag_entry as both will be treated seperately but same.
        if !tag_index_to_str.contains_key(&tag_entry.tag_index) {
            let tag_path_ptr = Memory::fix_pointer(tag_entry.tag_path_ptr);

            if let Ok(value) = memory.read_str(tag_path_ptr) {
                tag_index_to_str.insert(tag_entry.tag_index, value.to_string());
            }

            tag_index_to_tag_entry.insert(tag_entry.tag_index, tag_entry);
        }
    }

    Some(EngineSnapshot {
        object_header: object_manager,
        object_header_entries: object_pool_entries,
        object_entries: game_object_entries,
        player_header: player_manager,
        player_entries: player_pool_entries,
        player_globals: player_globals,
        tags: tag_index_to_str,
        tag_entries: tag_index_to_tag_entry
    })
}
