mod backend;
mod mods;
use crate::app::mods::components::Components;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    #[serde(skip)]
    menu: String,

    // picked_path
    #[serde(skip)]
    picked_path: Option<String>,

    // Components
    #[serde(skip)]
    components: Components,

    #[serde(skip)]
    api_addr: Option<String>,

    #[serde(skip)]
    api_port: Option<usize>,

    #[serde(skip)]
    api_secret: Option<String>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            menu: "History".to_owned(),
            picked_path: None,
            components: Components::default(),
            api_addr: None,
            api_port: None,
            api_secret: None,
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
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
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
                ui.menu_button("File", |ui| {
                    if ui.button("New").clicked() {
                        self.components = Components::default();
                        self.picked_path = None;
                    }
                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                });
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
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            /*
            show basic project window
            */
            if self.picked_path.is_some() {
                if self.api_addr.is_some() && self.api_secret.is_some() && self.api_port.is_some() {
                    let h = self.components.get_component_by_name("History").unwrap();
                    h.set_is_remote(true);
                    h.set_api_addr(self.api_addr.clone());
                    h.set_api_port(self.api_port.clone());
                    h.set_api_secret(self.api_secret.clone());
                }
                self.components.open("History", true);
                self.components.windows(ctx, &mut self.picked_path);
            } else if let Some(p) = self.components.get_component_by_name("Proxy") {
                p.show(
                    ui.ctx(),
                    &mut self.picked_path.is_some(),
                    &mut self.picked_path,
                );
                let is_remote = p.get_is_remote();
                let api_secret = p.get_api_secret();
                let api_addr = p.get_api_addr();
                let api_port = p.get_api_port();
                if is_remote {
                    self.api_secret = api_secret;
                    self.api_addr = api_addr;
                    self.api_port = api_port;
                }
            }

            /* check wether or not there's an inspector open or not */
            egui::warn_if_debug_build(ui);
        });
    }
}
