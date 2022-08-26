use crate::app::backend::background_proc::ProxyHandler;
use super::components::W;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
#[derive(Default)]
pub struct Proxy {
    is_open: bool,
    is_minimized: bool,
    is_spawned: bool,
    #[serde(skip)]
    handler: Option<ProxyHandler>,
}

impl W for Proxy {}

impl super::Component for Proxy {
    fn name(&self) -> &'static str {
        "Proxy"
    }

    fn show(&mut self, _ctx: &egui::Context, _open: &mut bool, _path: &Option<String>) {
        
    }
}

impl super::View for Proxy {
    fn ui(&mut self, ui: &mut egui::Ui, path: &Option<String>) {
        let is_spawned = format!("Running: {}", self.is_spawned);
        ui.label(is_spawned);
        ui.separator();
        if ui.button("stop").clicked(){
            self.stop();
        }
        ui.separator();
        if ui.button("start").clicked() && !self.is_spawned {
            self.start(path);
        }
        ui.separator();
        
    }
}

impl Proxy {
    fn start (&mut self, path: &Option<String>) {
        let path = if path.is_some() { path.clone().unwrap().to_string() } else { "/tmp/RPTProject".to_string() };
        self.handler = Some(ProxyHandler::start("sh", ["-c", &format!("./srv/rustyproxy-srv -d {} 2> {}/logs", &path, &path)]));
        self.is_spawned = true;  
    }

    fn stop (&mut self) {
        if let Some(h) = &mut self.handler {
            self.is_spawned = if h.kill() { false } else { true };
            self.handler = None;
        }
    }
}