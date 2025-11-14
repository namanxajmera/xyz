use crate::app::DepMgrApp;
use eframe::egui;

pub fn show_dashboard(ctx: &egui::Context, app: &mut DepMgrApp) {
    egui::CentralPanel::default().show(ctx, |_ui| {
        // Sidebar
        egui::SidePanel::left("sidebar")
            .resizable(true)
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.heading("Package Managers");
                ui.separator();

                // Manager filters
                for manager in &app.available_managers {
                    let is_selected = app.selected_managers.contains(manager);
                    if ui.checkbox(&mut app.selected_managers.contains(manager), manager.name()).clicked() {
                        if is_selected {
                            app.selected_managers.remove(manager);
                        } else {
                            app.selected_managers.insert(manager.clone());
                        }
                    }
                }

                ui.separator();
                ui.heading("Stats");
                
                let (total, outdated, orphaned) = app.stats();
                ui.label(format!("Total: {}", total));
                ui.label(format!("Outdated: {}", outdated));
                ui.label(format!("Orphaned: {}", orphaned));

                ui.separator();

                if ui.button("Refresh").clicked() {
                    app.request_refresh();
                }
            });

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Packages");

            // Search and filter bar
            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut app.search_query);
                
                ui.separator();
                
                ui.checkbox(&mut app.show_outdated_only, "Outdated Only");
                ui.checkbox(&mut app.show_orphaned_only, "Orphaned Only");
            });

            ui.separator();

            // Package table
            if app.is_scanning.load(std::sync::atomic::Ordering::Relaxed) {
                ui.centered_and_justified(|ui| {
                    ui.spinner();
                    ui.label("Scanning packages...");
                });
            } else {
                let filtered = app.filtered_packages();
                
                if filtered.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.label("No packages found");
                    });
                } else {
                    egui::ScrollArea::vertical()
                        .show(ui, |ui| {
                            egui::Grid::new("packages_grid")
                                .num_columns(6)
                                .spacing([10.0, 4.0])
                                .show(ui, |ui| {
                                    // Header
                                    ui.label("Name");
                                    ui.label("Manager");
                                    ui.label("Installed");
                                    ui.label("Latest");
                                    ui.label("Size");
                                    ui.label("Status");
                                    ui.end_row();

                                    // Rows
                                    for pkg in filtered {
                                        ui.label(&pkg.name);
                                        ui.label(pkg.manager.name());
                                        ui.label(&pkg.installed_version);
                                        
                                        if let Some(latest) = &pkg.latest_version {
                                            ui.label(latest);
                                        } else {
                                            ui.label("-");
                                        }

                                        // Show size if available
                                        if let Some(size) = pkg.size {
                                            ui.label(crate::utils::format_size(size));
                                        } else {
                                            ui.label("-");
                                        }

                                        if pkg.is_outdated {
                                            ui.label(egui::RichText::new("⚠️ Outdated").color(egui::Color32::from_rgb(255, 165, 0)));
                                        } else {
                                            ui.label(egui::RichText::new("✓ Up to date").color(egui::Color32::from_rgb(0, 200, 0)));
                                        }
                                        
                                        ui.end_row();
                                    }
                                });
                        });
                }
            }
        });
    });
}

