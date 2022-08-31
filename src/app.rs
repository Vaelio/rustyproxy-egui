mod backend;
mod mods;
use crate::app::backend::dbutils;
use crate::app::mods::components::Components;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    menu: String,

    // picked_path
    picked_path: Option<String>,

    // Components
    components: Components,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            menu: "History".to_owned(),
            picked_path: None,
            components: Components::default(),
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            ctx.request_repaint_after(std::time::Duration::from_secs_f32(1.0));
            egui::menu::bar(ui, |ui| {
                egui::widgets::global_dark_light_mode_switch(ui);
                ui.separator();
                if ui.button("🗀 Open").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        let path = path.display().to_string();
                        if dbutils::is_valid_project_path(&path) {
                            self.picked_path = Some(path);
                            self.components.open("History", true);
                        }
                    }
                }
                ui.separator();

                if let Some(picked_path) = &self.picked_path {
                    ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                        ui.horizontal(|ui| {
                            ui.monospace(picked_path);
                            ui.label("🗀 Project Path: ");
                            ui.separator();
                        });
                    });
                }
            });
        });
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("This program is yet in alpha. Feel free to contribute on");
                ui.hyperlink_to("Github", "https://github.com/vaelio/rustyproxy-egui");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    if let Some(p) = self.components.get_component_by_name("Proxy") {
                        p.ui(ui, &self.picked_path);
                    }
                });
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            if self.picked_path.is_some() {
                self.components.windows(ctx, &self.picked_path);
            } else {
                if ui.button("🗀 Open project").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        let path = path.display().to_string();
                        if dbutils::is_valid_project_path(&path) {
                            self.picked_path = Some(path);
                            self.components.open("History", true);
                        }
                    }
                }
            }

            /* check wether or not there's an inspector open or not */
            egui::warn_if_debug_build(ui);
        });
    }
}
