use crate::app::backend::batch_req;
use crate::app::backend::dbutils::HistLine;
use crate::app::mods::filter_cat::FilterCat;
use poll_promise::Promise;
use std::path::PathBuf;
use std::fs::File;
use std::io::{Read, Write};
use std::fmt::Write as fWrite;


#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub enum ActiveInspectorMenu {
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
pub struct Inspector {
    pub id: usize,
    pub source: String,
    pub request: String,
    pub response: String,
    pub modified_request: String,
    pub new_response: String,
    #[serde(skip)]
    pub response_promise: Option<Promise<Result<String, reqwest::Error>>>,
    pub ssl: bool,
    pub selected: Option<(usize, String, String, String, String, String)>,
    pub target: String,
    #[serde(skip)]
    pub active_window: ActiveInspectorMenu,
    #[serde(skip)]
    pub is_active: bool,
    #[serde(skip)]
    pub is_minimized: bool,
    #[serde(skip)]
    pub bf_payload: Vec<String>,
    pub bf_request: String,
    pub bf_results: Vec<batch_req::SuccessTuple>,
    #[serde(skip)]
    pub bf_promises: batch_req::VecPromiseType,
    #[serde(skip)]
    pub bf_payload_prepared: Vec<batch_req::Request>,
    #[serde(skip)]
    pub bf_current_page: usize,
    #[serde(skip)]
    pub bf_items_per_page: usize,
    #[serde(skip)]
    pub bf_filter_input: String,
    #[serde(skip)]
    pub bf_filter: Option<String>,
    #[serde(skip)]
    pub bf_filter_cat: Option<FilterCat>,
}


impl Inspector {
    pub fn from_histline(h: &HistLine) -> Self {
        Self {
            id: h.id(),
            source: h.remote_addr().to_string(),
            request: h.raw().to_string(),
            response: h.response().to_string(),
            modified_request: h.raw().to_string(),
            new_response: h.response().to_string(),
            bf_request: h.raw().to_string(),
            ssl: h.ssl(),
            target: h.host().to_string(),
            is_active: true,
            is_minimized: false,
            ..Default::default()
        }
    }
}


impl std::fmt::Debug for Inspector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error>  {
        write!(f, "Inspector #{} ", self.id)?;
        write!(f, "source: {} ", self.source)?;
        write!(f, "is_active: {} ", self.is_active)?;
        write!(f, "is_minimized: {} ", self.is_minimized)?;
        write!(f, "active_window: {:?} ", self.active_window)?;
        Ok(())
    }
}

#[macro_export]
macro_rules! inspector_ui {
    ($ui: expr, $w: expr, $inspected: expr) => {
        egui::menu::bar($ui, |ui| {
            if ui.button("☰ Default").clicked() {
                $inspected.active_window = ActiveInspectorMenu::Default;
            }
            ui.separator();
            if ui.button("☰ Modify").clicked() {
                $inspected.active_window = ActiveInspectorMenu::Repeater;
            }
            ui.separator();
            if ui.button("☰ Bruteforce").clicked() {
                $inspected.active_window = ActiveInspectorMenu::Intruder;
            }
            ui.separator();
            ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                ui.horizontal(|ui| {
                    if ui.button("x").clicked() {
                        $inspected.is_active = false;
                        ui.ctx().request_repaint();
                    }
                    ui.separator();
                    let bt = if $inspected.is_minimized { "+" } else { "-" };
                    if ui.button(bt).clicked() {
                        $inspected.is_minimized = !$inspected.is_minimized;
                        ui.ctx().request_repaint();
                    }
                    ui.separator();
                    ui.label(format!("{} <-> {}", &$inspected.source, &$inspected.target));
                    ui.label("💻 ");
                    ui.separator();
                    if $inspected.ssl {
                        ui.label("true");
                    } else {
                        ui.label("false");
                    }
                    ui.label("ssl: ");
                    ui.separator();
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        ui.label(format!("Viewing #{}", $inspected.id));
                    });
                });
            });
        });
        $ui.separator();
        if !$inspected.is_minimized {
            egui::ScrollArea::vertical().show($ui, |ui| {
                match $inspected.active_window {
                    ActiveInspectorMenu::Repeater => {
                        egui::menu::bar(ui, |ui| {
                            if ui.button("⚠ Reset").clicked() {
                                $inspected.modified_request =
                                    $inspected.request.to_string().replace('\r', "\\r\\n");
                                $inspected.new_response = $inspected.response.to_string();
                            }
                            ui.separator();
                            if ui.button("☰ Save Modified Request").clicked() {
                                if let Some(path) = rfd::FileDialog::new().save_file() {
                                    save_content_to_file(
                                        path,
                                        &$inspected.modified_request.replace("\\r\\n", "\r"),
                                    );
                                }
                            }
                            ui.separator();
                            if ui.button("☰ Copy as Curl").clicked() {
                                copy_as_curl(
                                    ui,
                                    &$inspected.modified_request.replace("\\r\\n", "\r"),
                                    $inspected.ssl,
                                    &$inspected.target,
                                );
                            }
                            ui.separator();
                            if ui.button("✉ Send").clicked() {
                                /* Parse request */
                                let request = $inspected.modified_request.replace("\\r\\n", "\r");
                                let method = request.split(' ').take(1).collect::<String>();
                                let uri = request.split(' ').skip(1).take(1).collect::<String>();
                                let url = format!(
                                    "{}://{}{}",
                                    if $inspected.ssl { "https" } else { "http" },
                                    $inspected.target,
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
                                $inspected.response_promise.get_or_insert_with(|| {
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
                                ui.ctx().request_repaint();
                            }
                            if $inspected.response_promise.is_some() {
                                if let Some(Ok(s)) = $inspected.response_promise.as_ref().unwrap().ready() {
                                    $inspected.new_response = s.to_string();
                                    $inspected.response_promise = None;
                                    ui.ctx().request_repaint();
                                }
                            }
                            
                        });
                        ui.separator();
                        code_edit_ui(ui, &mut $inspected.modified_request);
                        ui.separator();
                        code_view_ui(ui, &$inspected.new_response);
                    }
                    ActiveInspectorMenu::Default => {
                        egui::menu::bar(ui, |ui| {
                            if ui.button("☰ Save Request").clicked() {
                                if let Some(path) = rfd::FileDialog::new().save_file() {
                                    save_content_to_file(
                                        path,
                                        &$inspected.request.replace("\r\n", "\r"),
                                    );
                                }
                            }
                            ui.separator();
                            if ui.button("☰ Save All").clicked() {
                                if let Some(path) = rfd::FileDialog::new().save_file() {
                                    save_content_to_file(
                                        path,
                                        &serde_json::to_string(&$inspected).unwrap(),
                                    );
                                }
                            }
                            ui.separator();
                            if ui.button("☰ Copy as Curl").clicked() {
                                copy_as_curl(ui, &$inspected.request, $inspected.ssl, &$inspected.target);
                            }
                            ui.separator();
                        });
                        ui.separator();
                        code_view_ui(ui, &$inspected.request);
                        ui.separator();
                        code_view_ui(ui, &$inspected.response);
                    }
                    ActiveInspectorMenu::Intruder => {
                        egui::menu::bar(ui, |ui| {
                            if ui.button("⚠ Reset").clicked() {
                                $inspected.bf_request =
                                    $inspected.request.to_string().replace('\r', "\\r\\n");
                            }
                            ui.separator();
                            if ui.button("☰ Save Modified Request").clicked() {
                                if let Some(path) = rfd::FileDialog::new().save_file() {
                                    save_content_to_file(
                                        path,
                                        &$inspected.modified_request.replace("\\r\\n", "\r"),
                                    );
                                }
                            }
                            ui.separator();
                            if ui.button("✉ Send").clicked() {
                                /* Actually start bruteforcing */
                                let requests: Vec<String> = $inspected
                                    .bf_payload
                                    .iter()
                                    .map(|p| {
                                        $inspected
                                            .bf_request
                                            .replace("\\r\\n", "\r")
                                            .replace("$[PAYLOAD]$", p)
                                    })
                                    .collect();
                                $inspected.bf_payload_prepared = batch_req::Request::from_strings(
                                    requests,
                                    $inspected.ssl,
                                    $inspected.target.to_string(),
                                );
                                batch_req::BatchRequest::run(
                                    &$inspected.bf_payload_prepared,
                                    &mut $inspected.bf_promises,
                                );
                            }
                            ui.separator();
                            if ui.button("☰ Load Payloads from File").clicked() {
                                if let Some(path) = rfd::FileDialog::new().pick_file() {
                                    if let Some(payload) = load_content_from_file(path) {
                                        $inspected.bf_payload = payload
                                            .split('\n')
                                            .map(|v| v.trim_end().to_string())
                                            .collect::<Vec<String>>();
                                    }
                                }
                            }
                            ui.separator();
                            ui.label(format!("Number of request: {}", $inspected.bf_payload.len()));
                            ui.separator();
                        });
                        ui.separator();
                        code_edit_ui(ui, &mut $inspected.bf_request);
                        ui.separator();
                        tbl_ui_bf!(ui, $w, $inspected);
                    }
                }
            });
        }
    }
}

#[macro_export]
macro_rules! tbl_ui_bf {
    ($ui: expr, $w: expr, $inspected: expr) => {
        egui::ScrollArea::both()
        .show($ui, |ui| {
            let text_height = egui::TextStyle::Body.resolve(ui.style()).size;
            tbl_dyn_col!(
                ui,
                |mut body| {
                    $inspected.bf_promises.retain(|prom| {
                        if let Some(vr) = prom.ready() {
                            for r in vr {
                                match r {
                                    Ok((idx, version, status, headers, text)) => {
                                        $inspected.bf_results.push((
                                            *idx,
                                            version.to_string(),
                                            status.to_string(),
                                            headers.to_string(),
                                            text.to_string(),
                                        ))
                                    }
                                    Err((idx, e)) => $inspected.bf_results.push((
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
                        $inspected.bf_current_page,
                        $inspected.bf_items_per_page,
                        $inspected.bf_results.len(),
                        $inspected.bf_filter
                    );
                    for r in &$inspected.bf_results[range] {
                        let (idx, version, status, headers, text) = r;
                        let payload = $inspected.bf_payload[*idx].to_string();
                        body.row(text_height, |mut row| {
                            row!(
                                row,
                                {
                                    $inspected.selected = Some(
                                        (
                                            idx.to_owned(), 
                                            version.to_string(), 
                                            status.to_string(), 
                                            headers.to_string(), 
                                            text.to_string(), 
                                            payload.to_string(),
                                        )
                                    );
                                    $w.clicked = true;
                                },
                                idx.to_string(),
                                &payload,
                                text.len().to_string(),
                                status
                            );
                        });
                    }
                },
                $inspected.bf_current_page,
                $inspected.bf_items_per_page,
                $inspected.bf_results.len(),
                $inspected.bf_filter,
                $inspected.bf_filter_cat,
                &mut $inspected.bf_filter_input,
                Size::exact(60.0),
                Size::exact(400.0),
                Size::exact(60.0),
                Size::exact(60.0)
                //Size::exact(60.0)
            );
        });
    }
}


pub fn load_content_from_file(path: PathBuf) -> Option<String> {
    if let Ok(mut fd) = File::open(path) {
        let mut out = vec![];
        if fd.read_to_end(&mut out).is_ok() {
            return Some(String::from_utf8_lossy(&out).trim_end().to_string());
        }
    }
    None
}

pub fn save_content_to_file(path: PathBuf, content: &String) -> bool {
    if let Ok(mut fd) = File::create(path) {
        if write!(fd, "{}", content).is_ok() {
            return true;
        }
    }
    false
}

pub fn copy_as_curl(ui: &mut egui::Ui, content: &str, ssl: bool, target: &String) {
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

pub fn code_view_ui(ui: &mut egui::Ui, mut code: &str) {
    egui::TextEdit::multiline(&mut code)
        .font(egui::TextStyle::Monospace) // for cursor height
        .code_editor()
        .desired_rows(10)
        .desired_width(f32::INFINITY)
        .interactive(false)
        .lock_focus(false)
        .show(ui);
}

pub fn code_edit_ui(ui: &mut egui::Ui, code: &mut String) {
    egui::TextEdit::multiline(code)
        .font(egui::TextStyle::Monospace) // for cursor height
        .code_editor()
        .desired_rows(10)
        .desired_width(f32::INFINITY)
        .interactive(true)
        .lock_focus(true)
        .show(ui);
}