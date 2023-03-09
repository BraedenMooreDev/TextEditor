use std::fs::File;

use crate::file_handler::*;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    content: String,

    #[serde(skip)]
    is_settings_window_open: bool,

    #[serde(skip)]
    file: Option<File>, // this how you opt-out of serialization of a member
                        // #[serde(skip)]
                        // value: f32,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            content: "".to_owned(),
            is_settings_window_open: false,
            file: None,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            content,
            is_settings_window_open,
            file,
        } = self;

        egui::Window::new("Settings")
            .open(is_settings_window_open)
            .default_pos(egui::Pos2::new(20.0, 0.0))
            .show(ctx, |ui| {
                ui.label("AAAAaa");
            });

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            // Menu options with a tilde '~' are incomplete functionality.
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New File...").clicked() {
                        file_new(content);
                    }
                    ui.separator();
                    if ui.button("Open File").clicked() {
                        file_open(file, content);
                    }
                    ui.separator();
                    if ui.button("~ Save").clicked() { /* file_save(file, content); */ }
                    if ui.button("Save As...").clicked() {
                        file_saveas(file, content);
                    }
                    ui.separator();
                    if ui.button("Preferences").clicked() {
                        self.is_settings_window_open = true;
                    }
                    if ui.button("Exit").clicked() {
                        _frame.close();
                    }
                });
                ui.menu_button("Edit", |ui| {
                    if ui.button("~ Undo").clicked() {}
                    if ui.button("~ Redo").clicked() {}
                    ui.separator();
                    if ui.button("~ Cut").clicked() {}
                    if ui.button("~ Copy").clicked() {}
                    if ui.button("~ Paste").changed() {}
                });
            });
        });

        // egui::SidePanel::left("side_panel").show(ctx, |ui| {
        //     ui.heading("Side Panel");

        //     ui.horizontal(|ui| {
        //         ui.label("Write something: ");
        //         ui.text_edit_singleline(label);
        //     });

        //     ui.add(egui::Slider::new(value, 0.0..=10.0).text("value"));
        //     if ui.button("Increment").clicked() {
        //         *value += 1.0;
        //     }

        //     ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
        //         ui.horizontal(|ui| {
        //             ui.spacing_mut().item_spacing.x = 0.0;
        //             ui.label("powered by ");
        //             ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        //             ui.label(" and ");
        //             ui.hyperlink_to(
        //                 "eframe",
        //                 "https://github.com/emilk/egui/tree/master/crates/eframe",
        //             );
        //             ui.label(".");
        //         });
        //     });
        // });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            ui.with_layout(
                egui::Layout::centered_and_justified(egui::Direction::TopDown),
                |ui| {
                    ui.code_editor(content);
                },
            );
            egui::warn_if_debug_build(ui);
        });
    }
}
