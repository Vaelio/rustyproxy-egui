use egui::Context;
use std::collections::BTreeSet;

use super::Component;
use super::View;

pub trait W: View + Component {}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Components {
    #[serde(skip)]
    components: Vec<Box<dyn W>>,
    open: BTreeSet<String>,
}

impl Default for Components {
    fn default() -> Self {
        Self::from_components(vec![
            Box::new(super::history::History::default()),
            Box::new(super::proxy::Proxy::default()),
        ])
    }
}

impl Components {
    pub fn from_components(components: Vec<Box<dyn W>>) -> Self {
        let open = BTreeSet::new();

        Self { components, open }
    }

    pub fn get_component_by_name(&mut self, key: &'static str) -> Option<&mut Box<dyn W>> {
        for compo in &mut self.components {
            if compo.name() == key {
                return Some(compo);
            }
        }
        None
    }

    pub fn windows(&mut self, ctx: &Context, path: &Option<String>) {
        let Self { components, open } = self;
        for component in components {
            let mut is_open = open.contains(component.name());
            component.show(ctx, &mut is_open, path);
            set_open(open, component.name(), is_open);
        }
    }

    pub fn open(&mut self, key: &'static str, is_open: bool) {
        if is_open {
            if !self.open.contains(key) {
                self.open.insert(key.to_owned());
            }
        } else {
            self.open.remove(key);
        }
    }

    pub fn _is_component_open(&self, key: &'static str) -> bool {
        self.open.contains(key)
    }
}

fn set_open(open: &mut BTreeSet<String>, key: &'static str, is_open: bool) {
    if is_open {
        if !open.contains(key) {
            open.insert(key.to_owned());
        }
    } else {
        open.remove(key);
    }
}
