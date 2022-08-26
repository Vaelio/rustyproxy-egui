pub mod history;
pub mod proxy;
pub mod components;

pub trait Component {
    fn name(&self) -> &'static str;

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, path: &Option<String>);
}

pub trait View {
    fn ui(&mut self, ui: &mut egui::Ui, path: &Option<String>);
}