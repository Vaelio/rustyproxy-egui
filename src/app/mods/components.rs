use egui::Context;
use std::collections::BTreeSet;

use super::Component;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Components {
    #[serde(skip)]
    components: Vec<Box<dyn Component>>,
    open: BTreeSet<String>
}

impl Default for Components {
    fn default() -> Self {
        Self::from_components(vec![
            Box::new(super::history::History::default()),
        ])
    }
}

impl Components {
    pub fn from_components(components: Vec<Box<dyn Component>>) -> Self {
        let open = BTreeSet::new();
        
        Self { components, open }
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