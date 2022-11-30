#[macro_export]
macro_rules! proxy_ui {
    ($ui: expr, $w: expr, $api_addr_input: expr, $api_port_input: expr, $api_secret_input: expr) => {
        {
            $ui.vertical(|ui| {
                if ui.button("🗀 Open Local Project").clicked() {
                    todo!();
                }
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Api address:");
                    ui.text_edit_singleline($api_addr_input);
                });
                ui.horizontal(|ui| {
                    ui.label("Api port:");
                    ui.text_edit_singleline($api_port_input);
                });
                ui.horizontal(|ui| {
                    ui.label("Api Secret:");
                    ui.text_edit_singleline($api_secret_input);
                });
                ui.horizontal(|ui| {
                    /* connect */
                    if ui.button("Connect").clicked() || ui.input().key_pressed(egui::Key::Enter) {

                        $w.api_addr = if !$api_addr_input.is_empty() {
                            Some($api_addr_input.to_string())
                        } else {
                            None
                        };

                        $w.api_port = if !$api_port_input.is_empty() {
                            let p = $api_port_input.parse::<usize>();
                            Some(p.unwrap_or(8443))
                        } else {
                            None
                        };

                        $w.api_secret = if !$api_secret_input.is_empty() {
                            Some($api_secret_input.to_string())
                        } else {
                            None
                        };

                        if $w.api_addr.is_some() && $w.api_port.is_some() && $w.api_secret.is_some() {
                            $w.clicked = true;
                            $w.is_remote = true;
                        }
                    }
                });
            });
        }
        
    }
    
}