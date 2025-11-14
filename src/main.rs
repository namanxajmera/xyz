mod app;
mod managers;
mod models;
mod ui;
mod utils;

use app::DepMgrApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Dependency Manager")
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Dependency Manager",
        options,
        Box::new(|_cc| {
            // Initialize app
            let mut app = DepMgrApp::default();
            
            // Initialize async runtime and run initialization
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(app.initialize());

            Ok(Box::new(app))
        }),
    )
}

impl eframe::App for DepMgrApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle refresh requests
        self.handle_refresh();
        
        ui::show_dashboard(ctx, self);
    }
}
