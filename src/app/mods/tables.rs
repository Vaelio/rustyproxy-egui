#[macro_export]
macro_rules! tbl_dyn_col {
    ( $ui: expr, $closure: expr, $current_page: expr, $items_per_page: expr, $items_number: expr, $filter: expr, $filter_input: expr, $($cols:expr ),*) => {
        TableBuilder::new($ui)
            .striped(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .resizable(true)
            .scroll(false)
            .stick_to_bottom(false)
            $(.column($cols))*
            .body($closure);
        $ui.separator();
        egui::menu::bar($ui, |ui| {
            if $items_number > $items_per_page {
                let lbl = format!("Current page: {}", $current_page);
                ui.label(lbl);
                ui.label("⬌ Items per page: ");
                ui.add(
                    egui::Slider::new(
                        &mut $items_per_page,
                        (10 as usize)..=($items_number),
                    )
                    .logarithmic(true),
                );
            }
            ui.label("Filter by host: ");
            let response = ui.text_edit_singleline($filter_input);
            if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                if $filter_input != "" {
                    $filter = Some($filter_input.to_owned());
                } else {
                    $filter = None;
                }
            }
            ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                ui.horizontal(|ui| {
                    if ui.button(">").clicked() {
                        if $items_number - ($current_page * $items_per_page)
                            > $items_per_page
                        {
                            $current_page += 1;
                        }
                    }
                    if ui.button("<").clicked() {
                        if $current_page != 0 {
                            $current_page -= 1;
                        }
                    };
                    ui.monospace($items_number.to_string());
                    ui.label("Number of results: ");
                });
            });
        });
    };
}

#[macro_export]
macro_rules! paginate {
    ($current_page: expr, $items_per_page: expr, $items_number: expr, $filter: expr) => {{
        let mut range = Range {
            start: $current_page * $items_per_page,
            end: ($current_page + 1) * $items_per_page,
        };
        range.end = if range.end > $items_number || $filter.is_some() {
            $items_number
        } else {
            range.end
        };
        range
    }};
}

#[macro_export]
macro_rules! row {
    ($row: ident, $($cols:expr ),*) => {
        $(
            $row.col(|ui|{
                ui.add(egui::Label::new($cols).wrap(true));
            });
        )*
    }
}

#[macro_export]
macro_rules! filter {
    ($item: expr, $filter: expr) => {
        $filter.is_none() || $filter.is_some() && $item.contains::<&str>($filter.as_ref().unwrap())
    };
}
