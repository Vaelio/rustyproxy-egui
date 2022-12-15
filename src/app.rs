mod backend;
mod mods;

use mods::history::History;
use mods::inspector::{Inspector, ActiveInspectorMenu, load_content_from_file, save_content_to_file, copy_as_curl, code_view_ui, code_edit_ui};
use crate::{proxy_ui, history_ui, inspector_ui};
use crate::app::backend::apiutils;
use crate::{filter, paginate, row, tbl_dyn_col, tbl_ui_bf};
use crate::app::backend::batch_req;
use crate::app::mods::filter_cat::FilterCat;
use reqwest::header::HeaderMap;


use poll_promise::Promise;
use egui_extras::{Size, TableBuilder};
use std::ops::Range;

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
    windows: Vec<Window>,

    #[serde(skip)]
    api_addr: Option<String>,

    #[serde(skip)]
    api_port: Option<usize>,

    #[serde(skip)]
    api_secret: Option<String>,

    #[serde(skip)]
    api_addr_input: String,

    #[serde(skip)]
    api_port_input: String,

    #[serde(skip)]
    api_secret_input: String,

    #[serde(skip)]
    history: Option<History>,

    zoom_level: String
}

#[derive(Debug)]
struct Window {
    name: String,
    is_active: bool,
    wtype: Wtype,
    api_addr: Option<String>,
    api_port: Option<usize>,
    api_secret: Option<String>,
    clicked: bool,
    is_remote: bool,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            is_active: true,
            name: String::new(),
            wtype: Wtype::default(),
            api_addr: None,
            api_port: None,
            api_secret: None,
            clicked: false,
            is_remote: false,
        }
        
    }
}

impl Window {

}


#[derive(Debug)]
enum Wtype {
    Proxy,
    History,
    Inspector(Inspector)
}

impl Default for Wtype {
    fn default() -> Self {
        Wtype::Proxy
    }
}


impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            menu: "History".to_owned(),
            picked_path: None,
            windows: vec![Window { name: "Proxy".to_string(), wtype: Wtype::Proxy, ..Default::default()}],
            api_addr: None,
            api_port: None,
            api_secret: None,
            api_addr_input: String::new(),
            api_port_input: String::new(),
            api_secret_input: String::new(),
            history: None,
            zoom_level: String::from("1.0"),
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
                        self.windows = vec![Window::default()];
                        self.picked_path = None;
                    }
                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                });
                ui.separator();
                ui.label("Zoom: ");
                let resp = ui.text_edit_singleline(&mut self.zoom_level);
                if resp.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                    if let Ok(value) = self.zoom_level.parse::<f32>() {
                        ctx.set_pixels_per_point(value);
                    }
                }
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
            let mut windows_to_add = vec![];
            let mut windows_to_remove = vec![];
            for w in &mut self.windows {
                if w.is_active {
                    
                        match &mut w.wtype {
                            Wtype::Proxy => {
                                egui::Window::new(format!("Child {}", w.name)).show(ctx, |ui| {
                                    proxy_ui!(ui, w, &mut self.api_addr_input, &mut self.api_port_input, &mut self.api_secret_input);
                                    self.api_addr = w.api_addr.clone();
                                    self.api_port = w.api_port.clone();
                                    self.api_secret = w.api_secret.clone();
                                    if w.clicked {
                                        w.is_active = false;
                                        windows_to_add.push(
                                            Window{
                                                name: "History".to_string(), 
                                                wtype: Wtype::History, 
                                                api_addr: self.api_addr.clone(), 
                                                api_port: self.api_port.clone(), 
                                                api_secret: self.api_secret.clone(),
                                                clicked: false,
                                                is_active: true,
                                                is_remote: true
                                            }
                                        );
                                        self.history = Some(History::default());
                                        w.clicked = false;
                                    }
                                });
                            },
                            Wtype::History => {
                                egui::Window::new(format!("Child {}", w.name)).show(ctx, |ui| {
                                    history_ui!(ui, w, self.history.as_mut().unwrap());
                                    if w.clicked {
                                        let h = self.history.as_ref().unwrap().selected().unwrap();
                                        windows_to_add.push(
                                            Window {
                                                name: format!("Inspecting #{}", h.id()), 
                                                wtype: Wtype::Inspector(Inspector::from_histline(&h)), 
                                                api_addr: self.api_addr.clone(), 
                                                api_port: self.api_port.clone(), 
                                                api_secret: self.api_secret.clone(),
                                                clicked: false,
                                                is_active: true,
                                                is_remote: true
                                            }
                                        );
                                        w.clicked = false;
                                        self.history.as_mut().unwrap().selected = None;
                                    }
                                });
                            },
                            Wtype::Inspector(i) => {
                                egui::Window::new(format!("Child {}", w.name))
                                    .title_bar(false)
                                    .show(ctx, |ui| {
                                        inspector_ui!(ui, w, i);
                                        if w.clicked {
                                            let (idx, version, status, headers, text, payload) = i.selected.as_ref().unwrap();
                                            let request =
                                            i.bf_request.replace("$[PAYLOAD]$", &payload);
                                            let response = format!(
                                                "{} {}\r\n{}\r\n{}",
                                                version, status, headers, text
                                            );
                                            let ins = Inspector {
                                                id: *idx,
                                                source: "RustyProxy".to_string(),
                                                request: request.to_string(),
                                                response: response.to_string(),
                                                modified_request: request.replace('\r', "\\r\\n"),
                                                new_response: response,
                                                ssl: i.ssl,
                                                target: i.target.to_string(),
                                                is_active: true,
                                                bf_request: request.to_string().replace('\r', "\\r\\n"),
                                                ..Default::default()
                                            };
                                            windows_to_add.push(
                                                Window {
                                                    name: format!("Inspecting #{}", ins.id), 
                                                    wtype: Wtype::Inspector(ins), 
                                                    api_addr: self.api_addr.clone(), 
                                                    api_port: self.api_port.clone(), 
                                                    api_secret: self.api_secret.clone(),
                                                    clicked: false,
                                                    is_active: true,
                                                    is_remote: true
                                                }
                                            );
                                            w.clicked = false;
                                        }
                                        if !i.is_active {
                                            windows_to_remove.push(w.name.to_string())
                                        }
                                });
                                
                            }
                        };
                    
                }
            }

            self.windows.append(&mut windows_to_add);
            self.windows.retain(|w| !windows_to_remove.contains(&w.name));

            /* check wether or not there's an inspector open or not */
            egui::warn_if_debug_build(ui);
        });
    }
}
