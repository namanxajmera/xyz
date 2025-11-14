mod app;
mod managers;
mod models;
mod scanner;
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
            // Initialize app with default state
            let mut app = DepMgrApp::default();

            // Create a temporary runtime for initial setup
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                // Detect available package managers
                app.available_managers = crate::managers::detect_available_managers().await;
                println!(
                    "[DEBUG] Found {} package managers",
                    app.available_managers.len()
                );
                app.selected_managers = app.available_managers.iter().cloned().collect();
            });

            // Start the initial scan asynchronously (non-blocking)
            app.start_scan();

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
