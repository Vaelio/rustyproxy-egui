pub mod components;
pub mod history;
pub mod proxy;
pub mod tables;

pub trait Component {
    fn name(&self) -> &'static str;

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, path: &mut Option<String>);

    fn set_api_secret(&mut self, _: Option<String>) {}

    fn get_api_secret(&self) -> Option<String> {
        None
    }

    fn set_is_remote(&mut self, _: bool) {}

    fn get_is_remote(&self) -> bool {
        false
    }

    fn set_api_addr(&mut self, _: Option<String>) {}

    fn get_api_addr(&self) -> Option<String> {
        None
    }
}

pub trait View {
    fn ui(&mut self, ui: &mut egui::Ui, path: &mut Option<String>);
}
