use super::components::W;
use crate::app::mods::View;
use crate::app::backend::background_proc::ProxyHandler;
use crate::app::backend::dbutils;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Proxy {
    is_open: bool,
    is_minimized: bool,
    #[serde(skip)]
    handler: Option<ProxyHandler>,
    secret_input: String,
    secret: Option<String>,
    api_addr_input: String,
    api_addr: Option<String>,
    is_remote: bool
}

impl Default for Proxy {
    fn default() -> Self {
        Self {
            is_open: true,
            is_minimized: false,
            handler: None,
            secret_input: String::new(),
            secret: None,
            api_addr_input: String::new(),
            api_addr: None,
            is_remote: false,
        }
    }
}

impl W for Proxy {}

impl super::Component for Proxy {
    fn name(&self) -> &'static str {
        "Proxy"
    }
    
    fn show(&mut self, ctx: &egui::Context, _open: &mut bool, path: &mut Option<String>) {
        if self.is_open {
            egui::Window::new("Proxy")
            .scroll2([false, false])
            .resizable(true)
            .title_bar(false)
            .id(egui::Id::new("ProxySettings"))
            .show(ctx, |ui| {
                self.ui(ui, path)
            });
        }
        
    }

    fn get_api_secret(&self) -> Option<String> {
        self.secret.clone()
    }

    fn get_api_addr(&self) -> Option<String> {
        self.api_addr.clone()
    }

    fn get_is_remote(&self) -> bool {
        self.is_remote
    }
}

impl super::View for Proxy {
    fn ui(&mut self, ui: &mut egui::Ui, path: &mut Option<String>) {

        /* prepare layout */
        ui.vertical(|ui| {
            /* bar with title and stuff */
            egui::menu::bar(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Project Settings");
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                        let bt = if self.is_minimized { "+" } else { "-" };
                        if ui.button(bt).clicked() {
                            self.is_minimized = !self.is_minimized;
                            ui.ctx().request_repaint();
                        }
                        ui.separator();
                    });
                    
                    
                });
            });
            ui.separator();
            /* open local project */
            if ! self.is_minimized {
                ui.vertical(|ui| {
                    if ui.button("🗀 Open Local Project").clicked() {
                        if let Some(fpath) = rfd::FileDialog::new().pick_folder() {
                            let fpath = fpath.display().to_string();
                            if dbutils::is_valid_project_path(&fpath) {
                                let _ = path.insert(fpath);
                                self.is_open = false;
                            }
                        }
                    }
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("Api address:");
                        ui.text_edit_singleline(&mut self.api_addr_input);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Api Secret:");
                        ui.text_edit_singleline(&mut self.secret_input);
                    });
                    ui.horizontal(|ui| {
                        /* connect */
                        if ui.button("Connect").clicked() {
                            let _ = path.insert("Remote Project".to_string());
                            self.is_remote = true;
                            if !self.secret_input.is_empty() {
                                self.secret = Some(self.secret_input.to_owned());
                            }
                            if !self.api_addr_input.is_empty() {
                                self.api_addr = Some(self.api_addr_input.to_owned());
                            }
                            self.is_open = false;
                        }
                        /* start locally */
                        if ui.button("Start Local Proxy").clicked() {
                            let _ = path.insert("/tmp/RPTProject".to_string());
                            self.start(path);
                            self.is_open = false;
                        }
                    });
                    
                });
                
            }
            
        });
    }

}

impl Proxy {
    fn start(&mut self, path: &Option<String>) {
        let path = if path.is_some() {
            path.clone().unwrap()
        } else {
            "/tmp/RPTProject".to_string()
        };
        let secret = if self.secret.is_some() {
            self.secret.as_ref().unwrap().to_string()
        } else {
            "TEST".to_string()
        };
        let addr = if self.api_addr.is_some() {
            self.api_addr.as_ref().unwrap().to_string()
        } else {
            "127.0.0.1".to_string()
        };
        self.handler = Some(ProxyHandler::start("/bin/sh", ["-c", &format!("cd ./rustyproxy-srv/ && ./build.sh && ./target/release/rustyproxy-srv -a {} -d {} --secret {}", &addr, &path, &secret)]));
    }

    fn stop(&mut self) {
        if let Some(h) = &mut self.handler {
            h.kill();
            self.handler = None;
        }
    }
}
