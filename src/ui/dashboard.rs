use crate::app::DepMgrApp;
use eframe::egui;
use egui_extras::{Column, TableBuilder};

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
                
                let (total, outdated, unused) = app.stats();
                ui.label(format!("Total: {}", total));
                ui.label(format!("Outdated: {}", outdated));
                ui.label(format!("Unused: {}", unused));

                ui.separator();

                if ui.button("🔄 Refresh").clicked() {
                    app.request_refresh();
                }
                
                ui.separator();
                
                let (_, outdated, _) = app.stats();
                if outdated > 0 {
                    if ui.button(format!("⬆️ Update All ({})", outdated)).clicked() {
                        app.update_all_outdated();
                    }
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

            // Show scanning status
            let is_scanning = app.is_scanning.load(std::sync::atomic::Ordering::Relaxed);
            if is_scanning {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Scanning packages...");
                });
                ui.separator();
                // Request continuous repaints while scanning to show updates immediately
                ctx.request_repaint();
            }
            
            // Show update status
            let update_status = app.get_update_status();
            if !update_status.is_empty() {
                ui.horizontal(|ui| {
                    if update_status.contains("...") {
                        ui.spinner();
                    }
                    if update_status.starts_with("✓") {
                        ui.label(egui::RichText::new(&update_status).color(egui::Color32::from_rgb(0, 200, 0)));
                    } else if update_status.starts_with("✗") {
                        ui.label(egui::RichText::new(&update_status).color(egui::Color32::from_rgb(255, 0, 0)));
                    } else {
                        ui.label(&update_status);
                    }
                });
                ui.separator();
                ctx.request_repaint();
            }

            // Package table - show even while scanning
            let filtered = app.filtered_packages();
            
            if filtered.is_empty() && !is_scanning {
                ui.centered_and_justified(|ui| {
                    ui.label("No packages found");
                });
            } else if !filtered.is_empty() {
                // Wrap table in scroll area for both vertical and horizontal scrolling
                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        // Use resizable table instead of grid
                        TableBuilder::new(ui)
                            .striped(true)
                            .resizable(true)
                            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                            .column(Column::initial(100.0).at_least(60.0).resizable(true))  // Name
                            .column(Column::initial(80.0).at_least(60.0).resizable(true))   // Manager
                            .column(Column::initial(80.0).at_least(60.0).resizable(true))   // Installed
                            .column(Column::initial(80.0).at_least(60.0).resizable(true))   // Latest
                            .column(Column::initial(300.0).at_least(100.0).resizable(true)) // Description (wider)
                            .column(Column::initial(200.0).at_least(80.0).resizable(true))  // Usage (wider)
                            .column(Column::initial(80.0).at_least(60.0).resizable(true))   // Status
                            .column(Column::initial(100.0).at_least(80.0).resizable(true))  // Action
                    .header(20.0, |mut header| {
                        header.col(|ui| { ui.strong("Name"); });
                        header.col(|ui| { ui.strong("Manager"); });
                        header.col(|ui| { ui.strong("Installed"); });
                        header.col(|ui| { ui.strong("Latest"); });
                        header.col(|ui| { ui.strong("Description"); });
                        header.col(|ui| { ui.strong("Usage"); });
                        header.col(|ui| { ui.strong("Status"); });
                        header.col(|ui| { ui.strong("Action"); });
                    })
                    .body(|mut body| {
                        for pkg in filtered {
                            body.row(18.0, |mut row| {
                                row.col(|ui| { ui.label(&pkg.name); });
                                row.col(|ui| { ui.label(pkg.manager.name()); });
                                row.col(|ui| { ui.label(&pkg.installed_version); });
                                
                                row.col(|ui| {
                                    if let Some(latest) = &pkg.latest_version {
                                        ui.label(latest);
                                    } else {
                                        ui.label("-");
                                    }
                                });

                                // Description - no truncation, resizable column
                                row.col(|ui| {
                                    if let Some(desc) = &pkg.description {
                                        ui.label(desc);
                                    } else {
                                        ui.label("-");
                                    }
                                });

                                // Usage - show full folder names, resizable column
                                row.col(|ui| {
                                    if pkg.used_in.is_empty() {
                                        ui.label(egui::RichText::new("Unused").color(egui::Color32::from_rgb(200, 0, 0)));
                                    } else {
                                        // Extract folder names
                                        let folder_names: Vec<String> = pkg.used_in
                                            .iter()
                                            .filter_map(|path| {
                                                std::path::Path::new(path)
                                                    .file_name()
                                                    .and_then(|n| n.to_str())
                                                    .map(|s| s.to_string())
                                            })
                                            .collect();
                                        
                                        let display_text = folder_names.join(", ");
                                        ui.label(egui::RichText::new(display_text).color(egui::Color32::from_rgb(0, 150, 0)));
                                    }
                                });

                                // Status
                                row.col(|ui| {
                                    if pkg.is_outdated {
                                        ui.label(egui::RichText::new("⚠️ Outdated").color(egui::Color32::from_rgb(255, 165, 0)));
                                    } else {
                                        ui.label(egui::RichText::new("✓ Current").color(egui::Color32::from_rgb(0, 200, 0)));
                                    }
                                });
                                
                                // Action buttons
                                row.col(|ui| {
                                    ui.horizontal(|ui| {
                                        if pkg.is_outdated {
                                            let is_updating = app.is_updating(&pkg.name);
                                            if is_updating {
                                                ui.spinner();
                                            } else if ui.button("Update").clicked() {
                                                app.update_package(pkg.name.clone(), pkg.manager.clone());
                                            }
                                        }
                                        
                                        if ui.button("Remove").clicked() {
                                            app.uninstall_package(pkg.name.clone(), pkg.manager.clone());
                                        }
                                    });
                                });
                            });
                        }
                    });
                });
            }
        });
    });
}

