#![allow(dead_code)]
mod engine;
mod memory;

use std::ffi::OsStr;

use engine::{build_snapshot, object_type_string, Datum, EngineSnapshot};
use glow::HasContext;
use imgui::{Condition, Context, TableBgTarget, TableFlags, Ui};
use imgui_glow_renderer::{glow, AutoRenderer};
use imgui_sdl2_support::SdlPlatform;
use memory::Memory;
use sdl2::{
    event::Event,
    video::{GLProfile, Window},
};
use sysinfo::System;

static GREEN: [f32; 4] = [0.69, 0.87, 0.15, 1.0];
static RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
static ORANGE: [f32; 4] = [0.97, 0.7, 0.17, 1.0];
static DARK_GREY: [f32; 4] = [0.14, 0.14, 0.14, 1.0];
static WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

struct DrawContext {
    memory: Option<Memory>,
    virtual_address: String,
    target_index: u32
}

// Create a new glow context.
fn glow_context(window: &Window) -> glow::Context {
    unsafe {
        glow::Context::from_loader_function(|s| window.subsystem().gl_get_proc_address(s) as _)
    }
}

fn print_player(ui: &Ui, snapshot: &EngineSnapshot, player_index: u16) {
    if let Some(entry) = snapshot.player_entries[player_index as usize].as_ref() {
        let local_dead_player = &snapshot.player_globals.local_dead_players[player_index as usize];

        ui.text_colored(ORANGE, format!("Current Object Datum: {:#?}", entry.slave_unit_index));
        ui.text_colored(ORANGE, format!("Next Object Datum: {:#?}", local_dead_player));
        ui.text_colored(ORANGE, format!("Last Object Datum: {:#?}", entry.last_slave_unit_index));

        {
            let index = entry.slave_unit_index.get_index() as usize;
    
            let mut found = false;
            if snapshot.object_entries.get(index).is_some() && snapshot.object_entries[index].as_ref().is_some() {
                let game_object_entry = snapshot.object_entries[index].as_ref().unwrap();

                ui.text_colored(ORANGE, format!("Position: X: {:.4} Y: {:.4} Z: {:.4}", game_object_entry.position[0], game_object_entry.position[1], game_object_entry.position[2]));
                found = true;
            }
    
            if !found {
                ui.text_colored(ORANGE, format!("Position: None"));
            }
        }
    } else { 
        ui.text_colored(ORANGE, format!("Unit Handle: None"));
        ui.text_colored(ORANGE, format!("Position: None"));
    }

    let handle = &snapshot.player_globals.local_dead_players[player_index as usize];
    {
        let index = handle.get_index() as usize;

        let mut found = false;
        if snapshot.object_entries.get(index).is_some() && snapshot.object_entries[index].as_ref().is_some() {
            let game_object_entry = snapshot.object_entries[index].as_ref().unwrap();

            ui.text_colored(ORANGE, format!("Next Datum Position: X: {:.4} Y: {:.4} Z: {:.4}", game_object_entry.position[0], game_object_entry.position[1], game_object_entry.position[2]));
            found = true;
        }

        if !found {
            ui.text_colored(ORANGE, format!("Next Datum Position: None"));
        }
    }

    if let Some(_) = snapshot.player_entries[player_index as usize].as_ref() {
    } else { 
        ui.text_colored(ORANGE, format!("Last Unit Handle: None")) 
    }
}

fn draw(ui: &mut Ui, should_exit: &mut bool, draw_context: &mut DrawContext) {
    let memory = draw_context.memory.as_mut().unwrap();
    memory.refresh();
    let snapshot = build_snapshot(memory);

    // Do not render anything if the snapshot is invalid.
    let width = ui.io().display_size[0];
    let height = ui.io().display_size[1];

    let mut first_free_index = 0;
    if let Some(snapshot) = &snapshot {
        // Find the first free entry in the object list?
        // Sometimes the next_object_index in the object_pool_header is not consistent with the next free entry in the object pool ???????????
        for (index, entry) in snapshot.object_entries.iter().enumerate() {
            if entry.is_none() {
                first_free_index = index;
                break;
            }
        }
    }

    ui.main_menu_bar(|| {
        if let Some(token) = ui.begin_menu("File") {
            if ui.menu_item("Close") {
                *should_exit = true;
            };
            if ui.menu_item("Detach") {
                draw_context.memory = None;
            };
            token.end();
        }

        if let Some(snapshot) = &snapshot {
            ui.text(" | ");
            ui.text_colored(ORANGE, format!("Next Object Index: {} ({})", snapshot.object_header.next_index, first_free_index));
            ui.text_colored(ORANGE, format!("Next Object ID: {}", snapshot.object_header.next_id));
        }
    });    

    let main_window = ui.window("Objects")
        .size([width - 450.0, height - 20.0], Condition::Always)
        .position([0.0, 20.0], Condition::Always)
        .resizable(false)
        .collapsible(false)
        .begin();

    let players_window = ui.window("Players Globals")
        .size([450.0, height - 20.0], Condition::Always)
        .position([width - 450.0, 20.0], Condition::Always)
        .resizable(false)
        .collapsible(false)
        .begin();


    if let None = snapshot {
        return;
    }

    // Snapshot should always be present after this point.
    let snapshot = snapshot.unwrap();

    if let Some(players_window) = players_window {
        let p = &snapshot.player_globals;
        ui.text_colored(ORANGE, format!("Respawn Failure: {}",p.respawn_failure));
        ui.text_colored(ORANGE, format!("Are All Dead: {}", p.are_all_dead));
        ui.text_colored(ORANGE, format!("Input Disabled: {}", p.input_disabled));
        ui.text_colored(ORANGE, format!("Teleported: {}", p.teleported));

        for player in &snapshot.player_globals.local_players {
            if !player.is_invalid() {
                let player_index = player.get_index();
                ui.text(format!("-------------- Player {} --------------", player_index));
                print_player(ui, &snapshot, player_index);
            }
        }

        players_window.end();
    }

    if let Some(main_window) = main_window {
        if let Some(table) = ui.begin_table_with_flags("ObjectsTable", 8, TableFlags::SIZING_STRETCH_PROP) {
            ui.table_setup_column("");
            ui.table_setup_column("Datum");
            ui.table_setup_column("Index");
            ui.table_setup_column("ID");
            ui.table_setup_column("Player");
            ui.table_setup_column("Coordinates");
            ui.table_setup_column("Tag Name");
            ui.table_setup_column("Object Type");
            ui.table_headers_row();

            for index in 0..snapshot.object_header.max_entries as usize {
                ui.table_next_row();

                let is_row_valid = snapshot.object_entries[index].is_some();
                let identity = ui.push_id_usize(index);

                if is_row_valid {
                    let object_pool_entry = snapshot.object_header_entries[index].as_ref().unwrap();
                    let game_object_entry = snapshot.object_entries[index].as_ref().unwrap();
    
                    let datum_handle = Datum::from_parts(index as u16, object_pool_entry.id);
    
                    ui.table_set_column_index(0);
    
                    if ui.button(&"Set") {
                        draw_context.target_index = index as u32;
                    }
                    
                    if index == draw_context.target_index as usize {
                        ui.table_set_bg_color(TableBgTarget::ROW_BG0, DARK_GREY);
                    }
    
                    ui.table_next_column();
                    ui.text_colored(if first_free_index as usize == index { ORANGE } else { GREEN }, format!("{}", datum_handle.get_handle()));
    
                    ui.table_next_column();
                    ui.text_colored(if first_free_index as usize == index { ORANGE } else { GREEN }, format!("{}", index));
    
                    ui.table_next_column();
                    ui.text_colored(
                        if object_pool_entry.id == snapshot.object_header.next_id { ORANGE } else { WHITE }, 
                        format!("{:<5}", object_pool_entry.id )
                    );
    
                    ui.table_next_column();
                    if let Some(player_index) = snapshot.find_local_player_index_from_unit_index(index as u16) {
                        ui.text_colored(GREEN, format!("{}", player_index))
                    } else if let Some(local_dead_player_index) = snapshot.find_next_object_datum_player(datum_handle.clone()) {
                        ui.text_colored(RED, format!("{}", local_dead_player_index));
                    } else {
                        ui.text("");
                    }
    
                    ui.table_next_column();
                    let mut updated_position = game_object_entry.position.clone();
    
                    if ui.input_float3(&"POS", &mut updated_position).build() {
                        let manager = draw_context.memory.as_mut().unwrap();
                        let game_object_pointer = Memory::fix_pointer(object_pool_entry.object_address);
    
                        manager.write(game_object_pointer + 0xC, &updated_position.get(0).as_ref().expect("Could not write X position").to_le_bytes());
                        manager.write(game_object_pointer + 0xC + 0x4, &updated_position.get(1).as_ref().expect("Could not write Y position").to_le_bytes());
                        manager.write(game_object_pointer + 0xC + 0x4 + 0x4, &updated_position.get(2).as_ref().expect("Could not write Z position").to_le_bytes());
                    }
    
                    ui.table_next_column();
                    ui.text(snapshot.tags.get(&game_object_entry.tag_index).unwrap_or(&"UNKNOWN".to_string()).split("\\").last().unwrap());
    
                    ui.table_next_column();        
                    ui.text(object_type_string(object_pool_entry.data_type));
                } else {
                    ui.table_set_column_index(0);

                    if ui.button(&"Set") {
                        draw_context.target_index = index as u32;
                    }

                    if index == draw_context.target_index as usize {
                        ui.table_set_bg_color(TableBgTarget::ROW_BG0, DARK_GREY);
                    }

                    ui.table_next_column();
                    ui.text("");

                    ui.table_next_column();
                    ui.text_colored(if first_free_index as usize == index { ORANGE } else { RED }, format!("{}", index));

                    ui.table_next_column();
                    ui.text("Free");

                    ui.table_next_column();
                    ui.text("");

                    ui.table_next_column();
                    ui.text("");

                    ui.table_next_column();
                    ui.text("");

                    ui.table_next_column();
                    ui.text("");
                }

                identity.end();
            }

            table.end();
        }

        main_window.end();
    }
}


fn draw_attach(ui: &mut Ui, should_exit: &mut bool, draw_context: &mut DrawContext) {
    ui.main_menu_bar(|| {
        if let Some(token) = ui.begin_menu("File") {
            if ui.menu_item("Close") {
                *should_exit = true;
            };
            token.end();
        }
    });

    let width = ui.io().display_size[0];
    let height = ui.io().display_size[1];

    let attach_window = ui.window("Attach")
        .size([width, height - 20.0], Condition::Always)
        .position([0.0, 20.0], Condition::Always)
        .resizable(false)
        .collapsible(false)
        .begin();


    if let Some(attach_window) = attach_window {

        let sys = System::new_all();
        let processes: Vec<_> = sys.processes_by_exact_name(OsStr::new("xemu.exe")).collect();

        if processes.len() == 0 {
            ui.text("Could not find running instance of xemu.exe");
        } else if processes.len() > 1 {
            ui.text("Found multiple instances of xemu.exe running on the system. Please only have one instance running.");
        } else if processes.len() == 1 {
            ui.text("Found xemu.exe");

            let process = processes[0];

            ui.text(r#"Run (gpa2hva 0x0) in xemu.exe and put the result below."#);
            
            ui.input_text("Virtual Address to Physical Xbox Memory", &mut draw_context.virtual_address)
                .allow_tab_input(false)
                .chars_hexadecimal(true)
                .chars_noblank(true)
                .build();

            if ui.button("Set Virtual Address") {
                if let Ok(value) = usize::from_str_radix(&draw_context.virtual_address, 16) {
                    let mem = Memory::new(value, 67108864, process.pid().as_u32());
                    draw_context.memory = Some(
                        mem
                    );
                }
            }

    
        }

        attach_window.end();
    }

}

fn start() {
    // Setup draw context
    let mut draw_context = DrawContext {
        virtual_address: String::default(),
        memory: None,
        target_index: u32::MAX
    };


    /* */
    /* initialize SDL and its video subsystem */
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();

    /* hint SDL to initialize an OpenGL 3.3 core profile context */
    let gl_attr = video_subsystem.gl_attr();

    gl_attr.set_context_version(3, 3);
    gl_attr.set_context_profile(GLProfile::Core);

    /* create a new window, be sure to call opengl method on the builder when using glow! */
    let window = video_subsystem
        .window("Halo Object Viewer", 1600, 900)
        .opengl()
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    /* create a new OpenGL context and make it current */
    let gl_context = window.gl_create_context().unwrap();
    window.gl_make_current(&gl_context).unwrap();

    /* enable vsync to cap framerate */
    window.subsystem().gl_set_swap_interval(1).unwrap();

    /* create new glow and imgui contexts */
    let gl = glow_context(&window);

    /* create context */
    let mut imgui = Context::create();

    /* disable creation of files on disc */
    imgui.set_ini_filename(None);
    imgui.set_log_filename(None);

    /* setup platform and renderer, and fonts to imgui */
    imgui
        .fonts()
        .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);

    /* create platform and renderer */
    let mut platform = SdlPlatform::init(&mut imgui);
    let mut renderer = AutoRenderer::initialize(gl, &mut imgui).unwrap();

    /* start main loop */
    let mut event_pump = sdl.event_pump().unwrap();

    'main: loop {
        for event in event_pump.poll_iter() {
            /* pass all events to imgui platfrom */
            platform.handle_event(&mut imgui, &event);

            if let Event::Quit { .. } = event {
                break 'main;
            }
        }

        /* call prepare_frame before calling imgui.new_frame() */
        platform.prepare_frame(&mut imgui, &window, &event_pump);

        let ui = imgui.new_frame();
        let mut should_exit = false;
        
        /* create imgui UI here */
        
        if draw_context.memory.is_some() {
            draw(ui, &mut should_exit, &mut draw_context);
        } else {
            draw_attach(ui, &mut should_exit, &mut draw_context);
        }

        if should_exit {
            break 'main;
        }

        /* render */
        let draw_data = imgui.render();

        unsafe { renderer.gl_context().clear(glow::COLOR_BUFFER_BIT) };
        renderer.render(draw_data).unwrap();

        window.gl_swap_window();
    }
}

fn main() {
    start();
}
