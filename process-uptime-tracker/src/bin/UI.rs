use eframe::egui;
use std::fs::File;
use std::io::BufReader;
use std::collections::HashMap;

const SAVE_FILE: &str = "processes_uptime.json";

fn main() -> Result<(), eframe::Error> 
{
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Process Uptime",
        options,
        Box::new(|_cc| Ok(Box::new(ProcessUptimeApp::new()))),
    )
}

fn read_data() -> Result<HashMap<String, u64>, Box<dyn
 std::error::Error>>
{
    let file = File::open(SAVE_FILE)?;
    let reader = BufReader::new(file);
    let data: HashMap<String, u64> = serde_json::from_reader(reader)?;
    Ok(data)
}

struct ProcessUptimeApp 
{
    process_data: HashMap<String, u64>
}

impl ProcessUptimeApp
{
    fn new() -> Self
    {
        let process_data = read_data().unwrap_or_default();
        Self {process_data}
    }
}

fn format_time(uptime: u64) -> String
{
    if uptime >= 3600 {
        let hours = uptime / 3600;
        let minutes = (uptime % 3600) / 60;
        let seconds = uptime % 60;
        format!("{hours}h {minutes}m {seconds}s")
    }
    else if uptime >= 60
    {
        let minutes = uptime / 60;
        let seconds = uptime % 60;
        format!("{minutes}m {seconds}s")
    }
    else
    {
        format!("{uptime}s")
    }

}

impl eframe::App for ProcessUptimeApp
{
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame)
    {
        egui::CentralPanel::default().show(ctx, |ui| 
        {
            ui.heading("Process Uptimes");

            let mut process_entries: Vec<(&String, &u64)> = self.process_data.iter().collect();
            process_entries.sort_by_key(|&(_, uptime)| uptime);
            process_entries.reverse();

            for (name, uptime) in process_entries
            {
                ui.label(format!("{name}: {}", format_time(*uptime)));

                let size = (*uptime as f32).powf(0.5).min(750.0);

                let (response, rect) = ui.allocate_space(egui::vec2(size, 20.0));
                let painter = ui.painter();
                painter.rect_filled(rect, 5.0, egui::Color32::from_rgb(0, 0, 255));
            }
        });
    }
}
