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
                egui::ScrollArea::both()
                    .show(ui, |ui| {
                        egui::Grid::new("packages_grid")
                            .num_columns(9)
                            .spacing([10.0, 4.0])
                            .striped(true)
                            .show(ui, |ui| {
                                // Header
                                ui.strong("Name");
                                ui.strong("Manager");
                                ui.strong("Installed");
                                ui.strong("Latest");
                                ui.strong("Description");
                                ui.strong("Used In");
                                ui.strong("Usage");
                                ui.strong("Status");
                                ui.strong("Action");
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

                                    // Description (truncate if too long)
                                    if let Some(desc) = &pkg.description {
                                        let short_desc = if desc.len() > 50 {
                                            format!("{}...", &desc[..50])
                                        } else {
                                            desc.clone()
                                        };
                                        ui.label(short_desc).on_hover_text(desc);
                                    } else {
                                        ui.label("-");
                                    }
                                    
                                    // Used in directories
                                    if pkg.used_in.is_empty() {
                                        ui.label("-");
                                    } else {
                                        let dirs_text = if pkg.used_in.len() == 1 {
                                            pkg.used_in[0].clone()
                                        } else {
                                            format!("{} projects", pkg.used_in.len())
                                        };
                                        let full_list = pkg.used_in.join("\n");
                                        ui.label(dirs_text).on_hover_text(full_list);
                                    }
                                    
                                    // Usage status
                                    if pkg.used_in.is_empty() {
                                        ui.label(egui::RichText::new("❌ Unused").color(egui::Color32::from_rgb(200, 0, 0)));
                                    } else {
                                        ui.label(egui::RichText::new(format!("✓ {} proj", pkg.used_in.len())).color(egui::Color32::from_rgb(0, 150, 0)));
                                    }

                                    // Update status
                                    if pkg.is_outdated {
                                        ui.label(egui::RichText::new("⚠️ Outdated").color(egui::Color32::from_rgb(255, 165, 0)));
                                    } else {
                                        ui.label(egui::RichText::new("✓ Current").color(egui::Color32::from_rgb(0, 200, 0)));
                                    }
                                    
                                    // Update button
                                    if pkg.is_outdated {
                                        let is_updating = app.is_updating(&pkg.name);
                                        if is_updating {
                                            ui.spinner();
                                        } else if ui.button("Update").clicked() {
                                            app.update_package(pkg.name.clone(), pkg.manager.clone());
                                        }
                                    } else {
                                        ui.label("");
                                    }
                                    
                                    ui.end_row();
                                }
                            });
                    });
            }
        });
    });
}

