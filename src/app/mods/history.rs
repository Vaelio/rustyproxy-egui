use egui_extras::{Size, TableBuilder};
use std::collections::BTreeMap;
use crate::app::backend::dbutils;
use poll_promise::Promise;
use std::ops::Range;
use super::components::W;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct History {
    last_id: usize,

    #[serde(skip)]
    inspectors: Vec<Inspector>,

    #[serde(skip)]
    history: Vec<dbutils::HistLine>,

    #[serde(skip)]
    current_top_id: usize,

    is_minimized: bool,

    current_page: usize,

    items_per_page: usize,
}

impl Default for History {
    fn default() -> Self {
        Self {
            last_id: 0,
            inspectors: vec![],
            history: vec![],
            current_top_id: 0,
            is_minimized: false,
            current_page: 0,
            items_per_page: 10,
        }
    }
}


enum ActiveInspectorMenu {
    Default,
    Repeater,
    Intruder
}

struct Inspector {
    id: usize,
    request: String,
    response: String,
    modified_request: String,
    new_response: String,
    response_promise: Option<Promise<ehttp::Result<String>>>,
    ssl: bool,
    target: String,
    active_window: ActiveInspectorMenu,
    is_active: bool,
    is_minimized: bool,
}

impl W for History {}

impl super::Component for History {
    fn name(&self) -> &'static str {
        "History"
    }

    fn show(&mut self, ctx: &egui::Context, _open: &mut bool, path: &Option<String>) {
        if let Some(path) = path {
            egui::Window::new("History")
                .scroll2([true, false])
                .resizable(true)
                .title_bar(false)
                .default_width(1024.0)
                .anchor(egui::Align2::LEFT_TOP, [0.0, 0.0])
                .show(ctx, |ui| {
                    self.show_table(ui, path);
                });

            for mut inspector in &mut self.inspectors {

                if inspector.is_active {
                    egui::Window::new(format!("Viewing #{}", inspector.id))
                        .title_bar(false)
                        .collapsible(true)
                        .scroll2([true, true])
                        .default_width(1024.0)
                        .show(ctx, |ui| {
                            inspect(ui, &mut inspector);
                        });
                }
                
            }
        }
    }
}

impl super::View for History {
    fn ui(&mut self, _ui: &mut egui::Ui, _path: &Option<String>) {
        
    }
}

fn code_view_ui(ui: &mut egui::Ui, mut code: &str) {

    
    egui::TextEdit::multiline(&mut code)
        .font(egui::TextStyle::Monospace) // for cursor height
        .code_editor()
        .desired_rows(10)
        .desired_width(f32::INFINITY)
        .interactive(false)
        .lock_focus(false)
        .show(ui);

}


fn code_edit_ui(ui: &mut egui::Ui, code: &mut String) {

    
    egui::TextEdit::multiline(code)
        .font(egui::TextStyle::Monospace) // for cursor height
        .code_editor()
        .desired_rows(10)
        .desired_width(f32::INFINITY)
        .interactive(true)
        .lock_focus(true)
        .show(ui);

}


fn inspect(ui: &mut egui::Ui, inspected: &mut Inspector) {
    egui::menu::bar(ui, |ui| {
        if ui.button("☰ Default").clicked() {
            inspected.active_window = ActiveInspectorMenu::Default;
        }
        ui.separator();
        if ui.button("☰ Modify").clicked() {
            inspected.active_window = ActiveInspectorMenu::Repeater;
        }
        ui.separator();
        if ui.button("☰ Bruteforce").clicked() {
            inspected.active_window = ActiveInspectorMenu::Intruder;
        }
        ui.separator();
        ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
            ui.horizontal(|ui| {
                if ui.button("x").clicked() {
                    inspected.is_active = false;
                    ui.ctx().request_repaint();
                }
                ui.separator();
                let bt = if inspected.is_minimized { "+" } else { "-" };
                if ui.button(bt).clicked() {
                    inspected.is_minimized = !inspected.is_minimized;
                    ui.ctx().request_repaint();
                }
                ui.separator();
                ui.label(&inspected.target);
                ui.label("💻 ");
                ui.separator();
                if inspected.ssl {
                    ui.label("true");
                } else {
                    ui.label("false");
                }
                ui.label("ssl: ");
                ui.separator();
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.label(format!("Viewing #{}", inspected.id));
                });
            });
        });
    });
    ui.separator();
    if !inspected.is_minimized {
        egui::ScrollArea::vertical().show(ui, |ui| {
            match inspected.active_window {
                ActiveInspectorMenu::Repeater => {
                    egui::menu::bar(ui, |ui| {
                        if ui.button("⚠ Reset").clicked() {
                            inspected.modified_request = inspected.request.to_string();
                            inspected.new_response = inspected.response.to_string();
                        }
                        ui.separator();
                        if ui.button("✉ Send").clicked() {
                            /* TODO: Parse request */
                            let method = inspected.modified_request.split(" ").take(1).collect::<String>();
                            let uri = inspected.modified_request.split(" ").skip(1).take(1).collect::<String>();
                            let url = format!("{}://{}{}", if inspected.ssl { "https" } else { "http" }, inspected.target, uri);
                            let body = inspected.modified_request.split("\r\n\r\n").skip(1).take(1).collect::<String>().as_bytes().to_vec();
                            let mut headers = BTreeMap::new();
    
                            for header in inspected.modified_request.split("\r\n").skip(1).map_while(|x| if x.len() > 0 { Some(x) } else { None }).collect::<Vec<&str>>() {
                                let name = header.split(": ").take(1).collect::<String>();
                                let value = header.split(": ").skip(1).collect::<String>();
                                headers.insert(name, value);
                            }
    
                            //println!("method: {}\nuri: {}\nurl: {}\nbody: {:?}\nheaders: {:?}", method, uri, url, body, headers);
                            /* Actually send the request */
                            let ctx = ui.ctx();
                            let promise = inspected.response_promise.get_or_insert_with(|| {
                                // Begin download.
                                // We download the image using `ehttp`, a library that works both in WASM and on native.
                                // We use the `poll-promise` library to communicate with the UI thread.
                                let ctx = ctx.clone();
                                let (sender, promise) = Promise::new();
                                let request = ehttp::Request{
                                    method: method,
                                    url: url,
                                    body: body,
                                    headers: headers
                                };
                                ehttp::fetch(request, move |response| {
                                    if let Ok(r) = response {
                                        let headers_v: String = r.headers.iter().map(|(key, value)| format!("{}: {}\r\n", key, value)).collect();
                                        sender.send(
                                            Ok(
                                                format!("HTTP/1.1 {} {}\r\n{}\r\n{}", r.status, r.status_text, headers_v, String::from_utf8_lossy(&r.bytes).to_string())
                                            )
                                        );
                                        ctx.request_repaint();
                                    }
                                });
                                promise
                            });
    
                            if let Some(Ok(s)) = promise.ready() {
                                inspected.new_response = s.to_string();
                                inspected.response_promise = None;
                            }
                        }
                        ui.separator();
    
                    });
                    if let Some(p) = &inspected.response_promise {
                        if let Some(Ok(s)) = p.ready() {
                            inspected.new_response = s.to_string();
                            inspected.response_promise = None;
                            ui.ctx().request_repaint();
                        }
                    }
                    ui.separator();
                    code_edit_ui(ui, &mut inspected.modified_request);
                    ui.separator();
                    code_view_ui(ui, &mut inspected.new_response);
                },
                ActiveInspectorMenu::Default => {
                    code_view_ui(ui, &inspected.request);
                    ui.separator();
                    code_view_ui(ui, &inspected.response);
                },
                ActiveInspectorMenu::Intruder => {
                    egui::menu::bar(ui, |ui| {
                        if ui.button("⚠ Reset").clicked() {
                            /* TODO: reset intruder to original request */
                        }
                        if ui.button("✉ Send").clicked() {
                            /* TODO: Actually start bruteforcing */
                        }
                    });
                    ui.separator();
                    ui.label("Not yet implemented!");
                }
                
            }
        });
    }
    

}

impl History {
    
    fn tbl_ui(&mut self, ui: &mut egui::Ui) {
        let text_height = egui::TextStyle::Body.resolve(ui.style()).size;
        TableBuilder::new(ui)
            .striped(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Size::exact(40.0))
            .column(Size::exact(80.0))
            .column(Size::remainder().at_least(400.0).at_most(600.0))
            .column(Size::exact(60.0))
            .column(Size::exact(70.0))
            .column(Size::exact(90.0))
            .column(Size::exact(60.0))
            .resizable(true)
            .scroll(false)
            .stick_to_bottom(false)
            .body(|mut body| {
                let mut range = Range {
                    start: self.current_page*self.items_per_page,
                    end: (self.current_page+1)*self.items_per_page,
                };
                range.end = if range.end > self.history.len() { self.history.len() } else { range.end };
                for histline in &self.history[range] {
                    if histline.id > self.last_id {
                        self.last_id = histline.id;
                    }
                    body.row(text_height, |mut row| {
                        row.col(|ui| {
                            ui.label(histline.id.to_string());
                        });
                        row.col(|ui| {
                            ui.label(histline.method.to_owned());
                        });
                        row.col(|ui| {
                            ui.label(histline.uri.to_owned());
                        });
                        row.col(|ui| {
                            ui.label(histline.size.to_string());
                        });
                        row.col(|ui| {
                            ui.label(histline.status.to_string());
                        });
                        row.col(|ui| {
                            ui.label(histline.response_time.to_owned());
                        });
                        row.col(|ui| {
                            if ui.button("🔍").clicked() {
                                self.inspectors.push(Inspector{
                                    id: histline.id,
                                    request: histline.raw.to_string(),
                                    response: histline.response.to_string(),
                                    modified_request: histline.raw.to_string(),
                                    new_response: histline.response.to_string(),
                                    response_promise: None,
                                    ssl: histline.ssl,
                                    target: histline.host.to_string(),
                                    active_window: ActiveInspectorMenu::Default,
                                    is_active: true,
                                    is_minimized: false,
                                });
                            }
                        });
                    })
                }
            });
    }


    fn show_table(&mut self, ui: &mut egui::Ui, path: &String) {
        ui.vertical(|ui| {
            egui::menu::bar(ui, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                    ui.horizontal(|ui|{
                        let bt = if self.is_minimized { "+" } else { "-" };
                        if ui.button(bt).clicked() {
                            self.is_minimized = !self.is_minimized;
                            ui.ctx().request_repaint();
                        }
                        ui.separator();
                        let txt = format!("{} Queries", self.history.len());
                        ui.label(txt);
                        ui.separator();
                        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                            ui.label("History");
                        });
                        
                    });
                    
                });
            });
            
            if let Some(rows) = dbutils::get_new_from_last_id(self.last_id, path) {
                for row in rows {
                    self.history.insert(0, row);
                }
            }

            if !self.is_minimized {
                egui::ScrollArea::both()
                    .max_width(900.0)
                    .max_height(400.0)
                    .show(ui, |ui| {
                        self.tbl_ui(ui);
                });
                
            }
            
            

            ui.separator();
            egui::menu::bar(ui, |ui| {
                let lbl = format!("Current page: {}", self.current_page);
                ui.label(lbl);
                ui.label("⬌ Items per page: ");
                ui.add(
                    egui::Slider::new(&mut self.items_per_page, (10 as usize)..=(self.history.len()))
                );
                ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                    ui.horizontal(|ui| {
                        if ui.button(">").clicked() {
                            if self.history.len() - (self.current_page*self.items_per_page) > self.items_per_page {
                                self.current_page += 1;
                            }
                        }
                        if ui.button("<").clicked() {
                            if self.current_page != 0 {
                                self.current_page -= 1;
                            }
                        };
                    });
                    
                });
            });
        });
    }
}
