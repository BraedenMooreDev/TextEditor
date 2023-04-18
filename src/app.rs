use crate::file_handler::*;
use crate::spell_check::*;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::path::PathBuf;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    #[serde(skip)]
    running: bool,
    content: String,
    path: Option<PathBuf>,

    #[serde(skip)]
    file: Option<File>,

    #[serde(skip)]
    is_settings_window_open: bool,

    text_font_size: f32,
    spell_checker_on: bool,
    theme: egui::Visuals,

    #[serde(skip)]
    speller: Speller,

    #[serde(skip)]
    prev_content: String,

    #[serde(skip)]
    corrections: HashMap<String, String>,
    // this how you opt-out of serialization of a member
    // #[serde(skip)]
    // value: f32,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            running: false,
            content: "".to_owned(),
            path: None,
            file: None,
            is_settings_window_open: false,
            text_font_size: 14.0,
            spell_checker_on: true,
            theme: egui::Visuals::dark(),
            speller: Speller {
                letters: "".to_owned(),
                n_words: HashMap::new(),
            },
            prev_content: "".to_owned(),
            corrections: HashMap::new(),
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
            let state: TemplateApp =
                eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();

            let mut sty = (*cc.egui_ctx.style()).clone();
            for (_text_style, font_id) in sty.text_styles.iter_mut() {
                font_id.size = state.text_font_size;
            }
            cc.egui_ctx.set_style(sty);
            return state;
        }

        Default::default()
    }
}

fn init(ctx: &egui::Context, theme: &egui::Visuals, speller: &mut Speller, ) {
    *speller = Speller {
        letters: "abcdefghijklmnopqrstuvwxyz".to_string(),
        n_words: HashMap::new(),
    };

    let contents = fs::read_to_string("src/spell_check_training.txt")
        .expect("Something went wrong reading the file");

    speller.train(&contents);

    ctx.set_visuals(theme.clone());
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
            running,
            content,
            path,
            file,
            is_settings_window_open,
            text_font_size,
            spell_checker_on,
            theme,
            speller,
            prev_content,
            corrections,
        } = self;

        if !*running {
            *running = true;
            init(ctx, theme, speller);
        }

        // Set the title of the window to the name of the currently open file.
        // If the app just opened, we need to open the file of the stored path from the last time it ran.
        match path {
            Some(p) => {
                _frame.set_window_title(
                    ("Text Editor - ".to_string() + p.file_name().unwrap().to_str().unwrap())
                        .as_str(),
                );

                if file.is_none() {
                    path_open(path, file, content);
                }
            }
            None => {
                _frame.set_window_title("Text Editor - New File");
            }
        }

        egui::Window::new("Settings")
            .open(is_settings_window_open)
            .default_pos(egui::Pos2::new(20.0, 0.0))
            .show(ctx, |ui| {
                ui.add_space(10.0);

                ui.checkbox(spell_checker_on, "Use Spell Checker");

                ui.add_space(10.0);

                ui.label("Font size");
                let mut temp_font_size: String = text_font_size.to_string();
                ui.text_edit_singleline(&mut temp_font_size);
                *text_font_size = temp_font_size.parse::<f32>().unwrap_or(0.0);

                ui.add_space(10.0);
                ui.horizontal(|ui| {

                    if ui.selectable_label(*theme == egui::Visuals::dark(), "Dark").clicked() {

                        *theme = egui::Visuals::dark();
                    }
    
                    if ui.selectable_label(*theme == egui::Visuals::light(), "Light").clicked() {
    
                        *theme = egui::Visuals::light();
                    }
                });

                ui.add_space(10.0);

                if ui.button("Apply").clicked() {
                    let mut sty = (*ctx.style()).clone();
                    for (_text_style, font_id) in sty.text_styles.iter_mut() {
                        font_id.size = *text_font_size;
                    }
                    ctx.set_style(sty);
                    ctx.set_visuals(theme.clone());
                }
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
                        file_new(path, file, content);
                        *file = None;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Open File").clicked() {
                        file_open(path, file, content);
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Save").clicked() {
                        file_save(path, file, content);
                        ui.close_menu();
                    }
                    if ui.button("Save As...").clicked() {
                        file_saveas(path, file, content);
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Preferences").clicked() {
                        self.is_settings_window_open = true;
                        ui.close_menu();
                    }
                    if ui.button("Exit").clicked() {
                        _frame.close();
                    }
                });
                ui.menu_button("Edit", |ui| {
                    if ui.button("~ Undo").clicked() {}
                    if ui.button("~ Redo").clicked() {}
                    ui.separator();
                    if ui.button("~ Cut").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("~ Copy").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("~ Paste").changed() {
                        ui.close_menu();
                    }
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
                    let editor = ui.code_editor(content);

                    editor.context_menu(|ui| {
                        ui.label("Corrections:");

                        let curr = content.split_whitespace();

                        corrections.clear();

                        for word in curr.into_iter() {
                            let correction = speller.correct(&word.to_lowercase());
                            println!("{}: {}", word, correction);

                            if correction != word.to_lowercase() {
                                corrections.insert(word.to_owned(), correction);
                            }
                        }

                        *prev_content = content.clone();

                        for key in corrections.clone().keys() {
                            let val = corrections.get(key).unwrap();

                            if ui
                                .selectable_label(false, key.to_owned() + " > " + val)
                                .clicked()
                            {
                                *content = content.replace(key, val);
                                corrections.remove(key);
                                ui.close_menu();
                            }
                        }
                    });
                },
            );
            egui::warn_if_debug_build(ui);
        });
    }
}
