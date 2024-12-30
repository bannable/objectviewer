use std::{collections::HashMap, ffi::CStr};

use crate::memory::Memory;

use super::{datum::Datum, EntityManager};

// Halo 1 Xbox Retail
const HALO_OBJECT_POOL_HEADER_ADDR: u32 = 0x000B9370;
const HALO_PLAYER_POOL_HEADER_ADDR: u32 = 0x00213C50;

const HALO_TAG_HEADER_ADDR: u32 = 0x003A6000;
const HALO_PLAYER_GLOBALS_ADDR: u32 = 0x00214E00;
const HALO_GAME_GLOBALS_ADDR: u32 = 0x611D4;
const HALO_GAME_TIME_GLOBALS: u32 = 0x612E8;

// Sanity check constants
const DEAH: u32 = 1751474532;
const LIAT: u32 = 1952541036;
const RNCS: u32 = 1935896178;

// Halo Structs
const MAXIMUM_NUMBER_OF_LOCAL_PLAYERS: usize = 4;

#[derive(Debug)]
pub enum EngineError {
    TagHeader,
    GameGlobals,
    PlayerHeader,
    ObjectHeader,
}

#[derive(Debug)]
#[repr(C)]
pub struct GameOptions {
    pub unk_0: u32,
    pub unk_4: u16,
    pub difficulty: i16,
    pub random_seed: i32,
    pub map_name: [u8; 256],
}

impl GameOptions {
    pub fn is_valid(&self) -> bool {
        self.difficulty >= 0 && self.difficulty < 4
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct GameGlobals {
    pub map_loaded: u8,
    pub active: u8,
    pub players_double_speed: u8,
    pub map_loading: u8,
    pub map_load_progress: f32,
    pub game_options: GameOptions,
}

impl TryFrom<&Memory> for GameGlobals {
    type Error = EngineError;

    fn try_from(memory: &Memory) -> Result<Self, Self::Error> {
        let game_globals: GameGlobals = memory.read(HALO_GAME_GLOBALS_ADDR);
        if game_globals.game_options.is_valid() {
            Ok(game_globals)
        } else {
            Err(EngineError::GameGlobals)
        }
    }
}

impl GameGlobals {
    pub fn get_map_name(&self) -> String {
        if self.map_loaded == 1 {
            if let Ok(local_map_name) = CStr::from_bytes_until_nul(&self.game_options.map_name) {
                if let Ok(real_local_map_name) = local_map_name.to_str() {
                    return real_local_map_name.to_string();
                }
            }
        }
        String::default()
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct GameTimeGlobals {
    pub initalized: u8,
    pub active: u8,
    pub paused: u8,
    pub unk_3: [u8; 9],
    pub local_time: u32,
    pub elapsed: u16,
    pub unk_18: [u8; 6],
    pub speed: f32,
    pub leftover_dt: f32,
}

impl TryFrom<&Memory> for GameTimeGlobals {
    type Error = EngineError;

    fn try_from(memory: &Memory) -> Result<Self, Self::Error> {
        let game_time_globals: GameTimeGlobals = memory.read(HALO_GAME_TIME_GLOBALS);
        // Figure out how to validate this
        Ok(game_time_globals)
    }
}

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
    pub combined_pvs_local: [u8; 0x40],
}

impl TryFrom<&Memory> for PlayersGlobals {
    type Error = EngineError;

    fn try_from(memory: &Memory) -> Result<Self, Self::Error> {
        let player_globals: PlayersGlobals = memory.read(HALO_PLAYER_GLOBALS_ADDR);
        // Figure out how to validate this
        // TODO: Maybe check if the player count is less than the maximum number of players? - ban
        Ok(player_globals)
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct PlayerDataEntry {
    pub id: u16,
    pub local_player_index: u16,
    pub player_name: [u16; 12],
    pub unknown_1: [i32; 6],
    pub slave_unit_index: Datum,      // datum
    pub last_slave_unit_index: Datum, // datum
    pub unknown_2: [u8; 150],
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
    unknown_2: u32,
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
    footer: u32, // tags backwards
}

impl TagHeader {
    pub fn is_valid(&self) -> bool {
        self.footer == RNCS
    }
}

impl TryFrom<&Memory> for TagHeader {
    type Error = EngineError;

    fn try_from(memory: &Memory) -> Result<Self, Self::Error> {
        let tag_header: TagHeader = memory.read(HALO_TAG_HEADER_ADDR);
        if tag_header.is_valid() {
            Ok(tag_header)
        } else {
            Err(EngineError::TagHeader)
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct ObjectHeaderEntry {
    pub id: u16,
    pub unk_2: u8,
    pub data_type: u8,
    pub unknown_2: u16,
    pub data_sizeof: u16,
    pub object_address: u32,
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

type ObjectHeader = EntityManager<ObjectHeaderEntry>;

impl TryFrom<&Memory> for ObjectHeader {
    type Error = EngineError;

    fn try_from(memory: &Memory) -> Result<Self, Self::Error> {
        let object_header: ObjectHeader = memory.read(HALO_OBJECT_POOL_HEADER_ADDR);
        if object_header.is_valid() {
            Ok(object_header)
        } else {
            Err(EngineError::ObjectHeader)
        }
    }
}

type ObjectHeaderEntries = Vec<Option<ObjectHeaderEntry>>;
type ObjectEntries = Vec<Option<Object>>;

type PlayerHeader = EntityManager<PlayerDataEntry>;

impl TryFrom<&Memory> for PlayerHeader {
    type Error = EngineError;

    fn try_from(memory: &Memory) -> Result<Self, Self::Error> {
        let player_header: PlayerHeader = memory.read(HALO_PLAYER_POOL_HEADER_ADDR);
        if player_header.is_valid() {
            Ok(player_header)
        } else {
            Err(EngineError::PlayerHeader)
        }
    }
}

type PlayerEntries = Vec<Option<PlayerDataEntry>>;
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
        0 => "bipd",  // unit obje
        1 => "vehi",  // unit obje
        2 => "weap",  // item obje
        3 => "eqip",  // item obje
        4 => "garb",  // item obje
        5 => "proj",  // obje
        6 => "scen",  // obje
        7 => "mach",  // devi obje
        8 => "ctrl",  // devi obje
        9 => "lifi",  // devi obje
        11 => "ssce", // obje
        _ => "UNKNOWN",
    }
}

// TODO: This struct has a lot of fields to initialize, should implement a builder pattern
// Application
#[derive(Debug)]
pub struct EngineSnapshot {
    pub map_name: String,
    pub player_header: PlayerHeader,
    pub player_entries: PlayerEntries,
    pub object_header: ObjectHeader,
    pub object_header_entries: ObjectHeaderEntries,
    pub object_entries: ObjectEntries,
    pub player_globals: PlayersGlobals,
    pub game_globals: GameGlobals,
    pub game_time_globals: GameTimeGlobals,
    pub tags: HashMap<u32, String>,
    pub tag_entries: HashMap<u32, TagEntry>,
}

impl EngineSnapshot {
    // We only check the index here as the game does not safely check the ID
    // Otherwise, during some AUP we would not see the proper assignments for players.
    pub fn find_local_player_index_from_unit_index(&self, unit_index: u16) -> Option<u16> {
        for player_pool_entry in self.player_entries.iter() {
            if player_pool_entry.is_none() {
                continue;
            }
            let player_pool_entry = player_pool_entry.as_ref().unwrap();

            if player_pool_entry.slave_unit_index.get_index() == unit_index {
                return Some(player_pool_entry.local_player_index);
            }
        }

        None
    }

    pub fn find_next_object_datum_player(&self, object_handle: Datum) -> Option<usize> {
        for (player_array_index, player_object_handle) in
            self.player_globals.local_dead_players.iter().enumerate()
        {
            let player_index = player_object_handle.get_index();
            let object_index = object_handle.get_index();

            if player_index == object_index {
                return Some(player_array_index);
            }
        }

        None
    }

    fn read_object_entries(
        memory: &Memory,
        object_manager: &EntityManager<ObjectHeaderEntry>,
    ) -> Vec<Option<Object>> {
        let object_pool_entries = object_manager.read(memory);
        let mut game_object_entries: Vec<_> =
            (0..object_manager.max_entries).map(|_| None).collect();

        for index in 0..object_manager.capacity as usize {
            if let Some(object_entry) = &object_pool_entries[index] {
                let object_address = Memory::fix_pointer(object_entry.object_address);
                if object_address != 0 && object_address >= size_of::<ObjectListHeader>() as u32 {
                    if let Some(game_object) = Self::read_game_object(memory, object_address) {
                        game_object_entries[index] = Some(game_object);
                    }
                }
            }
        }

        game_object_entries
    }

    fn read_game_object(memory: &Memory, object_address: u32) -> Option<Object> {
        let object_list_header_ptr = object_address - size_of::<ObjectListHeader>() as u32;
        let object_list_header: ObjectListHeader = memory.read(object_list_header_ptr);

        if object_list_header.header_head == DEAH && object_list_header.header_tail == LIAT {
            Some(memory.read(object_address))
        } else {
            None
        }
    }

    fn read_tag_mappings(
        memory: &Memory,
        tag_header: &TagHeader,
    ) -> (HashMap<u32, String>, HashMap<u32, TagEntry>) {
        let mut tag_index_to_str = HashMap::new();
        let mut tag_index_to_tag_entry = HashMap::new();
        let tag_array_base_ptr = Memory::fix_pointer(tag_header.tag_array_ptr);

        for index in 0..tag_header.tag_count {
            let tag_entry_ptr = tag_array_base_ptr + (size_of::<TagEntry>() as u32 * index);
            let tag_entry: TagEntry = memory.read(tag_entry_ptr);

            if let std::collections::hash_map::Entry::Vacant(e) =
                tag_index_to_str.entry(tag_entry.tag_index)
            {
                if let Ok(value) = memory.read_str(Memory::fix_pointer(tag_entry.tag_path_ptr)) {
                    e.insert(value.to_string());
                }
                tag_index_to_tag_entry.insert(tag_entry.tag_index, tag_entry);
            }
        }

        (tag_index_to_str, tag_index_to_tag_entry)
    }
}

pub fn build_snapshot(memory: &Memory) -> Result<EngineSnapshot, EngineError> {
    let tag_header = TagHeader::try_from(memory)?;
    let game_globals = GameGlobals::try_from(memory)?;
    let map_name = game_globals.get_map_name();
    let object_header = ObjectHeader::try_from(memory)?;
    let player_header = PlayerHeader::try_from(memory)?;
    let player_globals = PlayersGlobals::try_from(memory)?;
    let game_time_globals = GameTimeGlobals::try_from(memory)?;
    let object_header_entries = object_header.read(memory);
    let player_entries = player_header.read(memory);
    let object_entries = EngineSnapshot::read_object_entries(memory, &object_header);
    let (tags, tag_entries) = EngineSnapshot::read_tag_mappings(memory, &tag_header);

    Ok(EngineSnapshot {
        map_name,
        object_header,
        object_header_entries,
        object_entries,
        player_header,
        player_entries,
        player_globals,
        game_globals,
        game_time_globals,
        tags,
        tag_entries,
    })
}
