use crate::file_handler::*;
use crate::spell_check::*;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::time::Duration;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct RustyLemon {
    #[serde(skip)] running: bool,
    content: String,
    path: Option<PathBuf>,

    #[serde(skip)] file: Option<File>,

    #[serde(skip)] show_settings_menu: bool,

    text_font_size: f32,
    auto_save_on: bool,
    auto_save_interval: u64,
    #[serde(skip)] show_close_confirmation: bool,
    #[serde(skip)] allowed_to_close: bool,
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

impl Default for RustyLemon {
    fn default() -> Self {
        Self {
            // Example stuff:
            running: false,
            content: "".to_owned(),
            path: None,
            file: None,
            show_settings_menu: false,
            text_font_size: 14.0,
            auto_save_on: true,
            auto_save_interval: 30,
            show_close_confirmation: false,
            allowed_to_close: false,
            spell_checker_on: false,
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

impl RustyLemon {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            let state: RustyLemon =
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

impl eframe::App for RustyLemon {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
        
        if self.auto_save_on {
            file_save(&mut self.path, &mut self.file, &mut self.content, false);
        }
    }

    fn auto_save_interval(&self) -> std::time::Duration {
        Duration::from_secs(self.auto_save_interval)
    }

    fn on_close_event(&mut self) -> bool {

        if self.auto_save_on {
            
            file_save(&mut self.path, &mut self.file, &mut self.content, true);
            true
        } else {
            self.show_close_confirmation = true;
            self.allowed_to_close
        }
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self {
            running,
            content,
            path,
            file,
            show_settings_menu,
            text_font_size,
            auto_save_on,
            auto_save_interval,
            show_close_confirmation,
            allowed_to_close,
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
                    ("Rusty Lemon - ".to_string() + p.file_name().unwrap().to_str().unwrap())
                        .as_str(),
                );

                if file.is_none() {
                    path_open(path, file, content);
                }
            }
            None => {
                _frame.set_window_title("Rusty Lemon - New File");
            }
        }

        if *show_settings_menu {
            egui::Window::new("Settings")
                .collapsible(false)
                .resizable(true)
                .constrain(true)
                .show(ctx, |ui| {
                    ui.add_space(15.0);

                    ui.horizontal(|ui| {
                        ui.label("Auto Save");
                        ui.checkbox(auto_save_on, "");
                    });
                    
                    if *auto_save_on {
                        ui.horizontal(|ui| {
                            ui.label("Interval [Seconds]");
                            ui.add(egui::Slider::new(auto_save_interval, 1..=120));
                        });
                    }

                    ui.add_space(15.0);

                    ui.horizontal(|ui| {
                        ui.label("Spell Checker");
                        ui.checkbox(spell_checker_on, "");
                    });
                    
                    ui.add_space(15.0);

                    ui.horizontal(|ui| {
                        ui.label("Font Size");
                        ui.add(egui::Slider::new(text_font_size, 8.0..=48.0));
                    });

                    ui.add_space(15.0);

                    if ui.button(if *theme == egui::Visuals::dark() { "Light Mode" } else { "Dark Mode"} ).clicked() {

                        if      *theme == egui::Visuals::dark()  { *theme = egui::Visuals::light(); } 
                        else if *theme == egui::Visuals::light() { *theme = egui::Visuals::dark(); }
                    };

                    ui.add_space(10.0);
                    ui.separator();

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {

                        if ui.button("Apply").clicked() {
                            let mut sty = (*ctx.style()).clone();
                            for (_text_style, font_id) in sty.text_styles.iter_mut() {
                                font_id.size = *text_font_size;
                            }
                            ctx.set_style(sty);
                            ctx.set_visuals(theme.clone());
                        }
                        
                        if ui.button("Close").clicked() {
                            *show_settings_menu = false;
                        }

                        if ui.button("Revert to Default").clicked() {
                            *text_font_size = 14.0;
                            *auto_save_on = true;
                            *auto_save_interval = 30;
                            *spell_checker_on = false;
                            *theme = egui::Visuals::dark();
                            *corrections = HashMap::new();
                        }
                    });
                    
                });
        }

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
                        file_save(path, file, content, true);
                        ui.close_menu();
                    }
                    if ui.button("Save As...").clicked() {
                        file_saveas(path, file, content);
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Preferences").clicked() {
                        self.show_settings_menu = true;
                        ui.close_menu();
                    }
                    if ui.button("Exit").clicked() {
                        _frame.close();
                    }
                });
            });
        });

        if *show_close_confirmation {
            // Show confirmation dialog:
            egui::Window::new("Any unsaved changes will be lost")
                .collapsible(false)
                .resizable(false)
                .constrain(true)
                .show(ctx, |ui| {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                        if ui.button("Cancel").clicked() {
                            *show_close_confirmation = false;
                        }
                        
                        if ui.button("Quit without Saving").clicked() {
                            *allowed_to_close = true;
                            _frame.close();
                        }

                        if ui.button("Save and Quit").clicked() {
                            file_save(path, file, content, true);
                            *allowed_to_close = true;
                            _frame.close();
                        }
                    });
                });
        }

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
