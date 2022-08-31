use egui_extras::{Size, TableBuilder};
use clipboard::{ClipboardContext, ClipboardProvider};
use super::components::W;
use crate::app::backend::dbutils;
use poll_promise::Promise;
use reqwest::header::HeaderMap;
use std::fs::File;
use std::io::Write;
use std::ops::Range;
use std::path::PathBuf;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct History {
    last_id: usize,

    inspectors: Vec<Inspector>,

    history: Vec<dbutils::HistLine>,

    #[serde(skip)]
    current_top_id: usize,

    is_minimized: bool,

    current_page: usize,

    items_per_page: usize,

    host_filter: Option<String>,

    host_filter_input: String,
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
            host_filter: None,
            host_filter_input: String::new(),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
enum ActiveInspectorMenu {
    Default,
    Repeater,
    Intruder,
}

impl Default for ActiveInspectorMenu {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
#[derive(Default)]
struct Inspector {
    id: usize,
    request: String,
    response: String,
    modified_request: String,
    new_response: String,
    #[serde(skip)]
    response_promise: Option<Promise<Result<String, String>>>,
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
                .id(egui::Id::new("History"))
                .default_width(1024.0)
                .anchor(egui::Align2::LEFT_TOP, [0.0, 0.0])
                .show(ctx, |ui| {
                    self.show_table(ui, path);
                });

            for mut inspector in &mut self.inspectors {
                if inspector.is_active {
                    egui::Window::new(format!("Viewing #{}", inspector.id))
                        .title_bar(false)
                        .id(egui::Id::new(format!("{}", inspector.id)))
                        .collapsible(true)
                        .scroll2([true, true])
                        .default_width(800.0)
                        .show(ctx, |ui| {
                            inspect(ui, &mut inspector);
                        });
                }
            }
        }
    }
}

impl super::View for History {
    fn ui(&mut self, _ui: &mut egui::Ui, _path: &Option<String>) {}
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
                        if ui.button("☰ Save Modified Request").clicked() {
                            if let Some(path) = rfd::FileDialog::new().save_file() {
                                save_content_to_file(path, &inspected.modified_request);
                            }
                        }
                        ui.separator();
                        if ui.button("☰ Copy as Curl").clicked() {
                            copy_as_curl(&inspected.modified_request, inspected.ssl, &inspected.target);
                        }
                        ui.separator();
                        if ui.button("✉ Send").clicked() {
                            /* TODO: Parse request */
                            let method = inspected.modified_request.split(" ").take(1).collect::<String>();
                            let uri = inspected.modified_request.split(" ").skip(1).take(1).collect::<String>();
                            let url = format!("{}://{}{}", if inspected.ssl { "https" } else { "http" }, inspected.target, uri);
                            let body = inspected.modified_request.split("\r\n\r\n").skip(1).take(1).collect::<String>().as_bytes().to_vec();
                            let mut headers = HeaderMap::new();
                            for header in inspected.modified_request.split("\r\n").skip(1).map_while(|x| if x.len() > 0 { Some(x) } else { None }).collect::<Vec<&str>>() {
                                let name = reqwest::header::HeaderName::from_bytes(header.split(": ").take(1).collect::<String>().as_bytes()).unwrap();
                                let value = reqwest::header::HeaderValue::from_bytes(header.split(": ").skip(1).collect::<String>().as_bytes()).unwrap();
                                headers.insert(name, value);
                            }

                            /* Actually send the request */
                            let ctx = ui.ctx();
                            let promise = inspected.response_promise.get_or_insert_with(|| {
                                // Begin download.
                                // We download the image using `ehttp`, a library that works both in WASM and on native.
                                // We use the `poll-promise` library to communicate with the UI thread.
                                let ctx = ctx.clone();
                                let (sender, promise) = Promise::new();

                                let cli = reqwest::blocking::Client::builder()
                                    .danger_accept_invalid_certs(true)
                                    .default_headers(headers)
                                    .build()
                                    .unwrap();

                                cli.request(reqwest::Method::from_bytes(&method.as_bytes()).unwrap(), url)
                                    .body(body)
                                    .send()
                                    .and_then(move |r| {
                                        let headers: String = r.headers().iter().map(|(key, value)| format!("{}: {}\r\n", key, value.to_str().unwrap())).collect();
                                        sender.send(
                                            Ok(
                                                format!("{:?} {} {}\r\n{}\r\n{}", r.version(), r.status().as_str(), r.status().canonical_reason().unwrap(), headers, r.text().unwrap())
                                            )
                                        );
                                        ctx.request_repaint();
                                        Ok(())
                                    }).unwrap();

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
                    egui::menu::bar(ui, |ui| {
                        if ui.button("☰ Save Request").clicked() {
                            if let Some(path) = rfd::FileDialog::new().save_file() {
                                save_content_to_file(path, &inspected.request);
                            }
                        }
                        ui.separator();
                        if ui.button("☰ Save All").clicked() {
                            if let Some(path) = rfd::FileDialog::new().save_file() {
                                save_content_to_file(
                                    path,
                                    &format!(
                                        "--- ID ---\n{}\n--- Target ---\n{}\n--- SSL ---\n{}\n--- Orignal Request ---\n{}\n--- Orignal Response ---\n{}\n--- Modified Request ---\n{}\n--- Modified Request Response ---\n{}\n\n--- END ---", 
                                        &inspected.id,
                                        &inspected.target,
                                        &inspected.ssl,
                                        &inspected.request,
                                        &inspected.response,
                                        &inspected.modified_request,
                                        &inspected.new_response)
                                );
                            }
                        }
                        ui.separator();
                        if ui.button("☰ Copy as Curl").clicked() {
                            copy_as_curl(&inspected.request, inspected.ssl, &inspected.target);
                        }
                        ui.separator();
                    });
                    ui.separator();
                    code_view_ui(ui, &inspected.request);
                    ui.separator();
                    code_view_ui(ui, &inspected.response);
                },
                ActiveInspectorMenu::Intruder => {
                    egui::menu::bar(ui, |ui| {
                        if ui.button("⚠ Reset").clicked() {
                            /* TODO: reset intruder to original request */
                        }
                        ui.separator();
                        if ui.button("☰ Save Modified Request").clicked() {
                            if let Some(path) = rfd::FileDialog::new().save_file() {
                                save_content_to_file(path, &inspected.modified_request);
                            }
                        }
                        ui.separator();
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
            .column(Size::exact(100.0))
            .column(Size::exact(60.0))
            .column(Size::exact(70.0))
            .column(Size::exact(90.0))
            .column(Size::exact(60.0))
            .resizable(true)
            .scroll(false)
            .stick_to_bottom(false)
            .body(|mut body| {
                let mut range = Range {
                    start: self.current_page * self.items_per_page,
                    end: (self.current_page + 1) * self.items_per_page,
                };
                range.end = if range.end > self.history.len() || self.host_filter.is_some() {
                    self.history.len()
                } else {
                    range.end
                };

                for histline in &mut self.history[range] {
                    if histline.id > self.last_id {
                        self.last_id = histline.id;
                    }
                    let mut f = "";
                    if let Some(filter) = &self.host_filter {
                        f = &filter;
                    }
                    let mut uri = histline.uri.to_owned();
                    if histline.uri.starts_with("http://") {
                        uri = format!("/{}", histline.uri.split("/").skip(3).collect::<String>());
                    }
                    if self.host_filter.is_none() || histline.host.contains(f) {
                        body.row(text_height, |mut row| {
                            row.col(|ui| {
                                ui.label(histline.id.to_string());
                            });
                            row.col(|ui| {
                                ui.label(histline.method.to_owned());
                            });
                            row.col(|ui| {
                                ui.label(uri.to_owned());
                            });
                            row.col(|ui| {
                                ui.label(histline.host.to_owned());
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
                                    self.inspectors.push(Inspector {
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
                    
                }
            });
    }

    fn show_table(&mut self, ui: &mut egui::Ui, path: &String) {
        ui.vertical(|ui| {
            egui::menu::bar(ui, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                    ui.horizontal(|ui| {
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
                    .max_width(1000.0)
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
                ui.add(egui::Slider::new(
                    &mut self.items_per_page,
                    (10 as usize)..=(self.history.len()),
                ).logarithmic(true));
                ui.label("Filter by host: ");
                let response = ui.add(egui::TextEdit::singleline(&mut self.host_filter_input).id(egui::Id::new("host_filter")));
                if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                    if self.host_filter_input != "" {
                        self.host_filter = Some(self.host_filter_input.to_owned());
                    } else {
                        self.host_filter = None;
                    }
                    
                }
                ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                    ui.horizontal(|ui| {
                        if ui.button(">").clicked() {
                            if self.history.len() - (self.current_page * self.items_per_page)
                                > self.items_per_page
                            {
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

fn save_content_to_file(path: PathBuf, content: &String) -> bool {
    if let Ok(mut fd) = File::create(path.display().to_string()) {
        if let Ok(_) = write!(fd, "{}", content) {
            return true;
        }
    }
    return false;
}

fn copy_as_curl(content: &String, ssl: bool, target: &String) {

    let method = content.split(" ").take(1).collect::<String>();
    let uri = content.split(" ").skip(1).take(1).collect::<String>();
    let url = format!("{}://{}{}", if ssl { "https" } else { "http" }, target, uri);
    let body = content.split("\r\n\r\n").skip(1).take(1).collect::<String>();

    let mut scurl = format!("curl '{}' -X '{}' --data '{}'", url, method, body);
    for header in content.split("\r\n").skip(1).map_while(|x| if x.len() > 0 { Some(x) } else { None }).collect::<Vec<&str>>() {
        scurl.push_str(&format!(" -H '{}'", &header));
    }

    
    let mut clipboard: ClipboardContext = ClipboardProvider::new().unwrap();
    clipboard.set_contents(scurl.to_string()).unwrap();
}
