use crate::app::backend::dbutils;
use poll_promise::Promise;


#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct History {
    last_id: usize,

    history: Vec<dbutils::HistLine>,

    #[serde(skip)]
    current_top_id: usize,

    pub current_page: usize,

    pub items_per_page: usize,

    pub filter: Option<String>,

    pub filter_input: String,

    #[serde(skip)]
    pub selected: Option<dbutils::HistLine>,

    #[serde(skip)]
    pub response_promise: Option<Promise<Result<String, reqwest::Error>>>,
}

impl Clone for History {
    fn clone(&self) -> Self {
        Self {
            last_id: self.last_id,
            history: self.history.clone(),
            current_top_id: self.current_top_id,
            current_page: self.current_page,
            items_per_page: self.items_per_page,
            filter: self.filter.clone(),
            filter_input: self.filter_input.clone(),
            selected: None,
            response_promise: None,
        }
    }
}

impl Default for History {
    fn default() -> Self {
        Self {
            last_id: 0,
            history: vec![],
            current_top_id: 0,
            current_page: 0,
            items_per_page: 10,
            filter: None,
            filter_input: String::new(),
            response_promise: None,
            selected: None,
        }
    }
}

impl History {
    pub fn selected(&self) -> Option<dbutils::HistLine> {
        self.selected.clone()
    }

    pub fn last_id(&self) -> usize {
        self.last_id
    }

    pub fn set_last_id(&mut self, id: usize) {
        self.last_id = id;
    }

    pub fn response_promise_mut(&mut self) -> &mut Option<Promise<Result<String, reqwest::Error>>> {
        &mut self.response_promise
    }

    pub fn reset_promise(&mut self) {
        self.response_promise = None;
    }

    pub fn history_mut(&mut self) -> &mut Vec<dbutils::HistLine> {
        &mut self.history
    }

    pub fn history(&self) -> &Vec<dbutils::HistLine> {
        &self.history
    }
}


#[macro_export]
macro_rules! history_ui {
    ($ui: expr, $w: expr, $history: expr) => {
        $ui.vertical(|ui|{
            let mut data_to_append = vec![];
            if $w.is_remote {
                let last_id = $history.last_id();
                let promise = $history.response_promise.get_or_insert_with(|| {
                    let url = format!(
                        "{}:{}",
                        $w.api_addr.clone().unwrap(),
                        $w.api_port.clone().unwrap()
                    );
                    let secret = $w.api_secret.clone().unwrap();
                    Promise::spawn_thread("api", move || {
                        apiutils::get_new_from_last_id(last_id, &url, &secret)
                    })
                });
                if let Some(p) = promise.ready() {
                    if let Ok(s) = p {
                        let rows = apiutils::parse_result(s.to_string());
                        for row in rows {
                            let new_id = if row.id() > last_id {
                                row.id()
                            } else {
                                last_id
                            };
                            $history.set_last_id(new_id);
                            data_to_append.insert(0, row);
                        }
                        $history.reset_promise();
                        ui.ctx().request_repaint();
                    } else {
                        $history.reset_promise();
                    }
                } 
            } else {
                todo!();
            }
            for h in data_to_append {
                $history.history_mut().insert(0, h.clone());
            }
            let text_height = egui::TextStyle::Body.resolve(ui.style()).size;
            let len = $history.last_id();
            
            tbl_dyn_col!(
                ui,
                |mut body| {
                    let range = paginate!(
                        $history.current_page,
                        $history.items_per_page,
                        len,
                        &$history.filter
                    );
                    let mut selected = None;
                    for item in &$history.history().clone()[range] {
                        if filter!(item.host(), &$history.filter) {
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
                                        selected = Some(item.clone());
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
                            })
                        }
                    }
                    if selected.is_some() {
                        $w.clicked = true;
                        $history.selected = selected;
                    }
                },
                $history.current_page,
                $history.items_per_page,
                len,
                $history.filter,
                &mut $history.filter_input,
                Size::exact(40.0),
                Size::exact(120.0),
                Size::exact(80.0),
                Size::exact(320.0),
                Size::exact(100.0),
                Size::exact(60.0),
                Size::exact(70.0),
                Size::exact(90.0)
            );
                    
        });
    }
}