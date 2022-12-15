#[macro_export]
macro_rules! tbl_dyn_col {
    ( $ui: expr, $closure: expr, $current_page: expr, $items_per_page: expr, $items_number: expr, $filter: expr, $filter_cat: expr, $filter_input: expr, $($cols:expr ),*) => {
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
            let menu = if $filter_cat.is_some(){
                format!("Filter by: {:?}", $filter_cat.as_ref().unwrap())
            }
            else {
                "Filter by".into()
            };
            ui.menu_button(menu, |ui| {
                if ui.button("Host").clicked() {
                    $filter_cat = Some(FilterCat::Host);
                }
                if ui.button("Code").clicked() {
                    $filter_cat = Some(FilterCat::Code);
                }
                if ui.button("Source").clicked() {
                    $filter_cat = Some(FilterCat::Source);
                }
                if ui.button("Path").clicked() {
                    $filter_cat = Some(FilterCat::Path);
                }
            });

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
    ($current_page: expr, $items_per_page: expr, $items_number: expr) => {{
        let mut range = Range {
            start: $current_page * $items_per_page,
            end: ($current_page + 1) * $items_per_page,
        };
        range.end = if range.end > $items_number {
            $items_number
        } else {
            range.end
        };
        range
    }};
}

#[macro_export]
macro_rules! row {
    ($row: ident, $closure: expr, $($cols:expr ),*) => {
        $(
            $row.col(|ui|{
                if ui.add(egui::Label::new($cols).wrap(true).sense(egui::Sense::click())).clicked() {
                    $closure;
                }
            });
        )*
    }
}

#[macro_export]
macro_rules! filter {
    ($item: expr, $filter: expr, $filter_cat: expr) => {
        match ($filter, $filter_cat){
            (Some(f),Some(FilterCat::Host)) => $item.host().contains::<&str>(f),
            (Some(f),Some(FilterCat::Code)) => $item.status() == f.parse::<usize>().unwrap(),
            (Some(f),Some(FilterCat::Source)) => $item.remote_addr().contains::<&str>(f),
            (Some(f),Some(FilterCat::Path)) => $item.uri().contains::<&str>(f),
            (None, _) => true,
            (Some(f), None) => $item.host().contains::<&str>(f),
        }
    };
}
