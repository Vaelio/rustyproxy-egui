use super::components::W;
use crate::app::backend::apiutils;
use crate::app::backend::batch_req;
use crate::app::backend::dbutils;
use crate::{filter, paginate, row, tbl_dyn_col};
use egui_extras::{Size, TableBuilder};
use poll_promise::Promise;
use reqwest::header::HeaderMap;
use std::fmt::Write as fWrite;
use std::fs::File;
use std::io::{Read, Write};
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
    source: String,
    request: String,
    response: String,
    modified_request: String,
    new_response: String,
    #[serde(skip)]
    response_promise: Option<Promise<Result<String, reqwest::Error>>>,
    ssl: bool,
    target: String,
    #[serde(skip)]
    active_window: ActiveInspectorMenu,
    #[serde(skip)]
    is_active: bool,
    #[serde(skip)]
    is_minimized: bool,
    #[serde(skip)]
    bf_payload: Vec<String>,
    bf_request: String,
    bf_results: Vec<batch_req::SuccessTuple>,
    #[serde(skip)]
    bf_promises: batch_req::VecPromiseType,
    #[serde(skip)]
    bf_payload_prepared: Vec<batch_req::Request>,
    #[serde(skip)]
    bf_current_page: usize,
    #[serde(skip)]
    bf_items_per_page: usize,
    #[serde(skip)]
    childs: Vec<Inspector>,
    #[serde(skip)]
    bf_filter_input: String,
    #[serde(skip)]
    bf_filter: Option<String>,
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
                }ui.separator();
                ui.label(format!("{} <-> {}", &inspected.source, &inspected.target));
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
                            inspected.modified_request =
                                inspected.request.to_string().replace('\r', "\\r\\n");
                            inspected.new_response = inspected.response.to_string();
                        }
                        ui.separator();
                        if ui.button("☰ Save Modified Request").clicked() {
                            if let Some(path) = rfd::FileDialog::new().save_file() {
                                save_content_to_file(
                                    path,
                                    &inspected.modified_request.replace("\\r\\n", "\r"),
                                );
                            }
                        }
                        ui.separator();
                        if ui.button("☰ Copy as Curl").clicked() {
                            copy_as_curl(
                                ui,
                                &inspected.modified_request.replace("\\r\\n", "\r"),
                                inspected.ssl,
                                &inspected.target,
                            );
                        }
                        ui.separator();
                        if ui.button("✉ Send").clicked() {
                            /* Parse request */
                            let request = inspected.modified_request.replace("\\r\\n", "\r");
                            let method = request.split(' ').take(1).collect::<String>();
                            let uri = request.split(' ').skip(1).take(1).collect::<String>();
                            let url = format!(
                                "{}://{}{}",
                                if inspected.ssl { "https" } else { "http" },
                                inspected.target,
                                uri
                            );
                            let body = request
                                .split("\r\n\r\n")
                                .skip(1)
                                .take(1)
                                .collect::<String>()
                                .as_bytes()
                                .to_vec();
                            let mut headers = HeaderMap::new();
                            for header in request
                                .split("\r\n")
                                .skip(1)
                                .map_while(|x| if !x.is_empty() { Some(x) } else { None })
                                .collect::<Vec<&str>>()
                            {
                                let name = reqwest::header::HeaderName::from_bytes(
                                    header.split(": ").take(1).collect::<String>().as_bytes(),
                                )
                                .unwrap();
                                let value = reqwest::header::HeaderValue::from_bytes(
                                    header.split(": ").skip(1).collect::<String>().as_bytes(),
                                )
                                .unwrap();
                                headers.insert(name, value);
                            }

                            /* Actually send the request */
                            let promise = inspected.response_promise.get_or_insert_with(|| {
                                Promise::spawn_thread("rq", move || {
                                    let cli = reqwest::blocking::Client::builder()
                                        .danger_accept_invalid_certs(true)
                                        .default_headers(headers)
                                        .redirect(reqwest::redirect::Policy::none())
                                        .build()
                                        .unwrap();

                                    cli.request(
                                        reqwest::Method::from_bytes(method.as_bytes()).unwrap(),
                                        url,
                                    )
                                    .body(body)
                                    .send()
                                    .map(move |r| {
                                        let headers: String = r
                                            .headers()
                                            .iter()
                                            .map(|(key, value)| {
                                                format!("{}: {}\r\n", key, value.to_str().unwrap())
                                            })
                                            .collect();
                                        format!(
                                            "{:?} {} {}\r\n{}\r\n{}",
                                            r.version(),
                                            r.status().as_str(),
                                            r.status().canonical_reason().unwrap(),
                                            headers,
                                            r.text().unwrap()
                                        )
                                    })
                                })
                            });

                            if let Some(Ok(s)) = promise.ready() {
                                inspected.new_response = s.to_string();
                                inspected.response_promise = None;
                                ui.ctx().request_repaint();
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
                    code_view_ui(ui, &inspected.new_response);
                }
                ActiveInspectorMenu::Default => {
                    egui::menu::bar(ui, |ui| {
                        if ui.button("☰ Save Request").clicked() {
                            if let Some(path) = rfd::FileDialog::new().save_file() {
                                save_content_to_file(
                                    path,
                                    &inspected.request.replace("\r\n", "\r"),
                                );
                            }
                        }
                        ui.separator();
                        if ui.button("☰ Save All").clicked() {
                            if let Some(path) = rfd::FileDialog::new().save_file() {
                                save_content_to_file(
                                    path,
                                    &serde_json::to_string(&inspected).unwrap(),
                                );
                            }
                        }
                        ui.separator();
                        if ui.button("☰ Copy as Curl").clicked() {
                            copy_as_curl(ui, &inspected.request, inspected.ssl, &inspected.target);
                        }
                        ui.separator();
                    });
                    ui.separator();
                    code_view_ui(ui, &inspected.request);
                    ui.separator();
                    code_view_ui(ui, &inspected.response);
                }
                ActiveInspectorMenu::Intruder => {
                    egui::menu::bar(ui, |ui| {
                        if ui.button("⚠ Reset").clicked() {
                            inspected.bf_request =
                                inspected.request.to_string().replace('\r', "\\r\\n");
                        }
                        ui.separator();
                        if ui.button("☰ Save Modified Request").clicked() {
                            if let Some(path) = rfd::FileDialog::new().save_file() {
                                save_content_to_file(
                                    path,
                                    &inspected.modified_request.replace("\\r\\n", "\r"),
                                );
                            }
                        }
                        ui.separator();
                        if ui.button("✉ Send").clicked() {
                            /* Actually start bruteforcing */
                            let requests: Vec<String> = inspected
                                .bf_payload
                                .iter()
                                .map(|p| {
                                    inspected
                                        .bf_request
                                        .replace("\\r\\n", "\r")
                                        .replace("$[PAYLOAD]$", p)
                                })
                                .collect();
                            inspected.bf_payload_prepared = batch_req::Request::from_strings(
                                requests,
                                inspected.ssl,
                                inspected.target.to_string(),
                            );
                            batch_req::BatchRequest::run(
                                &inspected.bf_payload_prepared,
                                &mut inspected.bf_promises,
                            );
                        }
                        ui.separator();
                        if ui.button("☰ Load Payloads from File").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_file() {
                                if let Some(payload) = load_content_from_file(path) {
                                    inspected.bf_payload = payload
                                        .split('\n')
                                        .map(|v| v.trim_end().to_string())
                                        .collect::<Vec<String>>();
                                }
                            }
                        }
                        ui.separator();
                        ui.label(format!("Number of request: {}", inspected.bf_payload.len()));
                        ui.separator();
                    });
                    ui.separator();
                    code_edit_ui(ui, &mut inspected.bf_request);
                    ui.separator();
                    tbl_ui_bf(ui, inspected);
                }
            }
        });
    }
}

fn tbl_ui_bf(ui: &mut egui::Ui, inspected: &mut Inspector) {
    egui::ScrollArea::both()
        .max_width(1000.0)
        .max_height(400.0)
        .show(ui, |ui| {
            let text_height = egui::TextStyle::Body.resolve(ui.style()).size;
            tbl_dyn_col!(
                ui,
                |mut body| {
                    inspected.bf_promises.retain(|prom| {
                        if let Some(vr) = prom.ready() {
                            for r in vr {
                                match r {
                                    Ok((idx, version, status, headers, text)) => {
                                        inspected.bf_results.push((
                                            *idx,
                                            version.to_string(),
                                            status.to_string(),
                                            headers.to_string(),
                                            text.to_string(),
                                        ))
                                    }
                                    Err((idx, e)) => inspected.bf_results.push((
                                        *idx,
                                        "SRVBUG".to_string(),
                                        e.to_string(),
                                        "SRVBUG".to_string(),
                                        "SRVBUG".to_string(),
                                    )),
                                }
                            }
                        }

                        prom.ready().is_none()
                    });
                    let range = paginate!(
                        inspected.bf_current_page,
                        inspected.bf_items_per_page,
                        inspected.bf_results.len(),
                        inspected.bf_filter
                    );
                    for r in &inspected.bf_results[range] {
                        let (idx, version, status, headers, text) = r;
                        let payload = inspected.bf_payload[*idx].to_string();
                        body.row(text_height, |mut row| {
                            row!(
                                row,
                                idx.to_string(),
                                &payload,
                                text.len().to_string(),
                                status
                            );

                            row.col(|ui| {
                                if ui.button("🔍").clicked() {
                                    let request =
                                        inspected.bf_request.replace("$[PAYLOAD]$", &payload);
                                    let response = format!(
                                        "{} {}\r\n{}\r\n{}",
                                        version, status, headers, text
                                    );

                                    let ins = Inspector {
                                        id: *idx,
                                        source:"".to_string(),
                                        request: request.to_string(),
                                        response: response.to_string(),
                                        modified_request: request.replace('\r', "\\r\\n"),
                                        new_response: response,
                                        ssl: inspected.ssl,
                                        target: inspected.target.to_string(),
                                        is_active: true,
                                        bf_request: request.to_string().replace('\r', "\\r\\n"),
                                        ..Default::default()
                                    };
                                    inspected.childs.push(ins);
                                }
                            });
                        });
                    }
                },
                inspected.bf_current_page,
                inspected.bf_items_per_page,
                inspected.bf_results.len(),
                inspected.bf_filter,
                &mut inspected.bf_filter_input,
                Size::exact(60.0),
                Size::exact(400.0),
                Size::exact(60.0),
                Size::exact(60.0),
                Size::exact(60.0)
            );
        });
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
                    if filter!(item.host, &self.filter) {
                        let uri = if item.uri.len() > 50 {
                            format!("{}[...]", &item.uri[..50])
                        } else {
                            item.uri.to_owned()
                        };
                        let host = if item.host.len() > 11 {
                            item.host[..11].to_string()
                        } else {
                            item.host.to_owned()
                        };
                        body.row(text_height, |mut row| {
                            row!(
                                row,
                                item.id.to_string(),
                                item.remote_addr.to_string(),
                                item.method.to_owned(),
                                uri,
                                host,
                                item.size.to_string(),
                                item.status.to_string(),
                                item.response_time.to_owned()
                            );
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
                            });
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
            Size::exact(90.0),
            Size::exact(40.0)
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
                        let url = format!("{}:{}", self.api_addr.clone().unwrap(), self.api_port.clone().unwrap());
                        let secret = self.api_secret.clone().unwrap();
                        Promise::spawn_thread("api", move || {
                            apiutils::get_new_from_last_id(last_id, &url, &secret)
                        })
                    });

                    if let Some(p) = promise.ready() {
                        if let Ok(s) = p {
                            let rows = apiutils::parse_result(s.to_string());
                            for row in rows {
                                self.last_id = if row.id > self.last_id {
                                    row.id
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
                            self.last_id = if row.id > self.last_id {
                                row.id
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

fn save_content_to_file(path: PathBuf, content: &String) -> bool {
    if let Ok(mut fd) = File::create(path) {
        if write!(fd, "{}", content).is_ok() {
            return true;
        }
    }
    false
}

fn copy_as_curl(ui: &mut egui::Ui, content: &str, ssl: bool, target: &String) {
    let method = content.split(' ').take(1).collect::<String>();
    let uri = content.split(' ').skip(1).take(1).collect::<String>();
    let url = format!("{}://{}{}", if ssl { "https" } else { "http" }, target, uri);
    let body = content
        .split("\r\n\r\n")
        .skip(1)
        .take(1)
        .collect::<String>();

    let mut scurl = format!("curl '{}' -X '{}' --data '{}'", url, method, body);
    for header in content
        .split("\r\n")
        .skip(1)
        .map_while(|x| if !x.is_empty() { Some(x) } else { None })
        .collect::<Vec<&str>>()
    {
        write!(scurl, " -H '{}'", &header).unwrap();
    }
    ui.output().copied_text = scurl;
}

fn load_content_from_file(path: PathBuf) -> Option<String> {
    if let Ok(mut fd) = File::open(path) {
        let mut out = vec![];
        if fd.read_to_end(&mut out).is_ok() {
            return Some(String::from_utf8_lossy(&out).trim_end().to_string());
        }
    }
    None
}
