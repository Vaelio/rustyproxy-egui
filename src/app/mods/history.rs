use super::components::W;
use crate::app::backend::apiutils;
use crate::app::backend::dbutils;
use crate::app::mods::inspector::{Inspector, inspect};
use crate::{filter, paginate, row, tbl_dyn_col};
use egui_extras::{Size, TableBuilder};
use poll_promise::Promise;
use std::ops::Range;

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

    filter: Option<String>,

    filter_input: String,

    is_remote: bool,

    #[serde(skip)]
    api_secret: Option<String>,

    #[serde(skip)]
    api_addr: Option<String>,

    #[serde(skip)]
    api_port: Option<usize>,

    #[serde(skip)]
    response_promise: Option<Promise<Result<String, reqwest::Error>>>,
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
            filter: None,
            filter_input: String::new(),
            is_remote: false,
            response_promise: None,
            api_addr: None,
            api_port: None,
            api_secret: None,
        }
    }
}



impl W for History {}

impl super::Component for History {
    fn name(&self) -> &'static str {
        "History"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, path: &mut Option<String>) {
        if *open {
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

                for inspector in &mut self.inspectors {
                    if inspector.is_active {
                        for child in &mut inspector.childs {
                            if child.is_active {
                                egui::Window::new(format!("Viewing Bruteforcer #{}", child.id))
                                    .title_bar(false)
                                    .id(egui::Id::new(format!("Bruteforcer {}", child.id)))
                                    .collapsible(true)
                                    .scroll2([true, true])
                                    .default_width(800.0)
                                    .show(ctx, |ui| {
                                        inspect(ui, child);
                                    });
                            }
                        }
                        egui::Window::new(format!("Viewing #{}", inspector.id))
                            .title_bar(false)
                            .id(egui::Id::new(format!("{}", inspector.id)))
                            .collapsible(true)
                            .scroll2([true, true])
                            .default_width(800.0)
                            .show(ctx, |ui| {
                                inspect(ui, inspector);
                            });
                    }
                }
            }
        }
    }

    fn set_is_remote(&mut self, b: bool) {
        self.is_remote = b;
    }

    fn set_api_addr(&mut self, a: Option<String>) {
        self.api_addr = a;
    }

    fn set_api_port(&mut self, p: Option<usize>) {
        self.api_port = p;
    }

    fn set_api_secret(&mut self, s: Option<String>) {
        self.api_secret = s;
    }
}

impl super::View for History {
    fn ui(&mut self, _ui: &mut egui::Ui, _path: &mut Option<String>) {}
}

impl History {
    fn tbl_ui(&mut self, ui: &mut egui::Ui) {
        let text_height = egui::TextStyle::Body.resolve(ui.style()).size;
        tbl_dyn_col!(
            ui,
            |mut body| {
                let range = paginate!(
                    self.current_page,
                    self.items_per_page,
                    self.history.len(),
                    self.filter
                );
                for item in &self.history[range] {
                    if filter!(item.host(), &self.filter) {
                        let uri = if item.uri().len() > 50 {
                            format!("{}[...]", &item.uri()[..50])
                        } else {
                            item.uri().to_owned()
                        };
                        let host = if item.host().len() > 11 {
                            item.host()[..11].to_string()
                        } else {
                            item.host().to_owned()
                        };
                        body.row(text_height, |mut row| {
                            row!(
                                row,
                                {
                                    let i = Inspector {
                                        id: item.id(),
                                        source: item.remote_addr().to_string(),
                                        request: item.raw().to_string(),
                                        response: item.response().to_string(),
                                        modified_request: item
                                            .raw()
                                            .to_string()
                                            .replace('\r', "\\r\\n"),
                                        new_response: item.response().to_string(),
                                        target: item.host().to_string(),
                                        bf_request: item.raw().to_string().replace('\r', "\\r\\n"),
                                        is_active: true,
                                        ssl: item.ssl(),
                                        ..Default::default()
                                    };
                                    self.inspectors.push(i);
                                },
                                item.id().to_string(),
                                item.remote_addr().to_string(),
                                item.method().to_owned(),
                                uri,
                                host,
                                item.size().to_string(),
                                item.status().to_string(),
                                item.response_time().to_owned()
                            );
                            /*
                            row.col(|ui| {
                                if ui.button("🔍").clicked() {
                                    let i = Inspector {
                                        id: item.id,
                                        source: item.remote_addr.to_string(),
                                        request: item.raw.to_string(),
                                        response: item.response.to_string(),
                                        modified_request: item
                                            .raw
                                            .to_string()
                                            .replace('\r', "\\r\\n"),
                                        new_response: item.response.to_string(),
                                        target: item.host.to_string(),
                                        bf_request: item.raw.to_string().replace('\r', "\\r\\n"),
                                        is_active: true,
                                        ..Default::default()
                                    };
                                    self.inspectors.push(i);
                                }
                            });*/
                        })
                    }
                }
            },
            self.current_page,
            self.items_per_page,
            self.history.len(),
            self.filter,
            &mut self.filter_input,
            Size::exact(40.0),
            Size::exact(120.0),
            Size::exact(80.0),
            Size::exact(320.0),
            Size::exact(100.0),
            Size::exact(60.0),
            Size::exact(70.0),
            Size::exact(90.0)
            //Size::exact(40.0)
        );
    }

    fn show_table(&mut self, ui: &mut egui::Ui, path: &str) {
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
            match self.is_remote {
                true => {
                    let promise = self.response_promise.get_or_insert_with(|| {
                        let last_id = self.last_id;
                        let url = format!(
                            "{}:{}",
                            self.api_addr.clone().unwrap(),
                            self.api_port.clone().unwrap()
                        );
                        let secret = self.api_secret.clone().unwrap();
                        Promise::spawn_thread("api", move || {
                            apiutils::get_new_from_last_id(last_id, &url, &secret)
                        })
                    });

                    if let Some(p) = promise.ready() {
                        if let Ok(s) = p {
                            let rows = apiutils::parse_result(s.to_string());
                            for row in rows {
                                self.last_id = if row.id() > self.last_id {
                                    row.id()
                                } else {
                                    self.last_id
                                };
                                self.history.insert(0, row);
                            }
                            self.response_promise = None;
                            ui.ctx().request_repaint();
                        } else {
                            self.response_promise = None;
                        }
                    }
                }
                false => {
                    if let Some(rows) = dbutils::get_new_from_last_id(self.last_id, path) {
                        for row in rows {
                            self.last_id = if row.id() > self.last_id {
                                row.id()
                            } else {
                                self.last_id
                            };
                            self.history.insert(0, row);
                        }
                    }
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
        });
    }
}






