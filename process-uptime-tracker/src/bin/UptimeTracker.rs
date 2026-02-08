#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::thread;
use sysinfo::{Pid, System, ProcessesToUpdate};
use std::fs;
use std::collections::HashMap;
use std::time::{Instant, Duration};
use windows::{
    Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowThreadProcessId,
    },
};
use tray_icon::{TrayIconBuilder, menu::{Menu, MenuEvent, MenuItem}, TrayIconEvent, MouseButton};
use std::process::Command;
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ProcessData
{
    name: String,
    uptime: u64,
}

const SAVE_FILE: &str = "processes_uptime.json";



fn main() -> Result<(), Box<dyn std::error::Error>>
{

    let tray_menu = Menu::new();
    let open_stats_item = MenuItem::new("Open Stats", true, None);
    tray_menu.append(&open_stats_item).unwrap();
    let quit_item = MenuItem::new("Quit", true, None);
    tray_menu.append(&quit_item).unwrap();

    let icon = load_icon(std::path::Path::new("icon.png"));
    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("Process Uptime Tracker")
        .with_icon(icon)
        .build()
        .unwrap();



    let mut data = if is_file_empty(SAVE_FILE) 
    {
        HashMap::new()
    }
    else
    {
        read_data()?
    };

    
    let delay = Duration::from_secs(1);
    let mut last_tick = Instant::now();

    let mut sys = System::new();
    let mut current_window = get_active_window_name(&sys).unwrap_or((0, String::from("unknown"))).1;

    loop 
    {
        sys.refresh_processes(ProcessesToUpdate::All, true);

        let now = Instant::now();
        let elapsed = now.duration_since(last_tick);
        last_tick = now;

        let Some((pid, title)) = get_active_window_name(&sys)
        else
        {
            thread::sleep(delay);
            continue;
        };

        let previous_uptime = *data.get(&title).unwrap_or(&0);
        let uptime = previous_uptime + elapsed.as_secs();

        let new_window = get_active_window_name(&sys).unwrap_or((0, String::from("unknown"))).1;

        if current_window != new_window
        {
            current_window = new_window;
        }


        data.insert(title.clone(), uptime);

        write_data(&data)?;        

        println!("{} {} {}s", pid, title, uptime);

        if let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == quit_item.id() {
                std::process::exit(0);
            }
            else if event.id == open_stats_item.id()
            {
                Command::new(ui_path()).spawn().ok();
            }
        }

        if let Ok(event) = TrayIconEvent::receiver().try_recv()
        {
            match event
            {
                TrayIconEvent::Click {button: MouseButton::Left, ..} =>
                {
                    Command::new(ui_path()).spawn().ok();
                }
                _ => {}
            }
        }

        thread::sleep(delay);        
    }
}

fn write_data(data: &HashMap<String, u64>) -> Result<(), Box<dyn std::error::Error>>
{
    let file = File::create(SAVE_FILE)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, data)?;

    Ok(())
}

fn read_data() -> Result<HashMap<String, u64>, Box<dyn
 std::error::Error>>
{
    let file = File::open(SAVE_FILE)?;
    let reader = BufReader::new(file);
    let data: HashMap<String, u64> = serde_json::from_reader(reader)?;
    println!("Read: {:?}", data);

    Ok(data)
}

fn get_active_window_name(sys: &System) -> Option<(u32, String)>
{
    unsafe {
        let hwnd = GetForegroundWindow();

        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));

        let process = sys.process(Pid::from_u32(pid))?;

        let name = process.name();
        Some((pid, name.to_string_lossy().to_string()))
    }
}

fn is_file_empty(file: &str) -> bool
{
    match fs::metadata(file) {
        Ok(metadata) => metadata.len() == 0,
        Err(_) => true,
    }
}

fn load_icon(path: &std::path::Path) -> tray_icon::Icon
{
    let (icon_rgba, icon_width, icon_height) =
    {
        let image = image::open(path).expect("Failed to open icon").into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to create icon")
}

fn ui_path() -> PathBuf
{
    std::env::current_exe().ok().and_then(|p| p.parent().map(|dir| dir.join("UI.exe"))).unwrap_or_else(|| PathBuf::from("UI.exe"))
}