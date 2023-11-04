use std::{array, collections::BTreeMap};

use egui::{CollapsingHeader, Color32, FontFamily, FontId, RichText, TextStyle};
use log::{info, warn};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct BinaryGeneratorWeb {
    // Example stuff:
    label: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,

    #[serde(skip)] // シャットダウンしても値を保持しない
    digit: u32,

    #[serde(skip)] // シャットダウンしても値を保持しない
    zoom_level: f32,

    #[serde(skip)] // シャットダウンしても値を保持しない
    default_text_styles: Option<BTreeMap<TextStyle, FontId>>,

    #[serde(skip)]
    binary: Option<Vec<u8>>,
}

impl Default for BinaryGeneratorWeb {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "0FFF".to_owned(),
            value: 2.7,
            digit: 8,
            zoom_level: 2.0,
            default_text_styles: None,
            binary: None,
        }
    }
}

impl BinaryGeneratorWeb {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // 日本語フォントに対応させる
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "Source Han Code JP".to_owned(),
            egui::FontData::from_static(include_bytes!("../SourceHanCodeJP.ttc")),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "Source Han Code JP".to_owned());
        cc.egui_ctx.set_fonts(fonts);

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        let mut se = Self::default();
        se.default_text_styles = Some(cc.egui_ctx.style().text_styles.clone());
        se
    }
}

impl eframe::App for BinaryGeneratorWeb {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let (zoom_delta, screen_rect) = ctx.input(|i| (i.zoom_delta(), i.screen_rect()));

        self.zoom_level = self.zoom_level + (zoom_delta - 1.0);
        // info!("{}: {}", zoom_delta, self.zoom_level);
        // ctx.set_pixels_per_point(self.zoom_level);
        // ctx.fonts_mut(|f| {
        //     if let Some(f) = f.as_mut() {
        //         f.
        //     }
        // });
        let zoom_level = self.zoom_level.clone();
        if self.default_text_styles == None {
            self.default_text_styles = Some(ctx.style().text_styles.clone());
        }
        let default_text_style = self.default_text_styles.clone().unwrap();
        ctx.style_mut(move |style| {
            let mut text_style = default_text_style;
            for (
                _style,
                FontId {
                    size,
                    family: _family,
                },
            ) in &mut text_style
            {
                *size = *size * zoom_level.clone();
            }
            style.text_styles = text_style;
        });

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.style_mut().text_styles = self.default_text_styles.clone().unwrap();

            // set ui size
            // Apply the zoom level to your UI elements.
            // This is just an example and may not work depending on your UI setup.
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
                {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            _frame.close();
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("Binary Generator Web");

            ui.horizontal(|ui| {
                ui.label("Write Binary: ");
                ui.text_edit_singleline(&mut self.label);
                #[cfg(target_arch = "wasm32")]
                {
                    if ui
                        .button("📋")
                        .on_hover_text("Copy to clipboard.")
                        .clicked()
                    {
                        if let Some(v) = self.binary.clone() {
                            // Copy binary data to clipboard
                            let _task = wasm_bindgen_futures::spawn_local(async move {
                                {
                                    let window = web_sys::window().expect("window"); // { obj: val };
                                    let nav = window.navigator().clipboard();
                                    match nav {
                                        Some(a) => {
                                            let blob = js_sys::Uint8Array::from(v.as_slice());
                                            let blob =
                                                web_sys::Blob::new_with_u8_array_sequence(&blob)
                                                    .unwrap();
                                            // blob.type_()
                                            change_blob_type(&blob, "text/plain");
                                            // let mut clipboard_array = js_sys::Array::new();
                                            let clipboard_array = js_sys::Array::new();
                                            // let item_obj = Object::new();

                                            let clipboard_item = web_sys::ClipboardItem::from(
                                                js_sys::wasm_bindgen::JsValue::from(blob),
                                            );
                                            clipboard_array.push(&clipboard_item);
                                            log::debug!("{:?}", clipboard_item);
                                            web_sys::console::dir(&clipboard_array);

                                            // let p = a.write(&clipboard_array);
                                            // let _result = wasm_bindgen_futures::JsFuture::from(p)
                                            //     .await
                                            //     .expect("clipboard populated");
                                            // info!("clippyboy worked");
                                        }
                                        None => {
                                            warn!("failed to copy clippyboy");
                                        }
                                    };
                                }
                            });
                            // ui.output_mut(|o| o.copied_text = v);
                        }
                    }
                }
            });

            // 2文字ごとに16進数の文字列として処理する
            let mut array: Option<Vec<u8>> = Some(Vec::new());
            let mut label = String::new();
            for (i, c) in self.label.to_ascii_lowercase().chars().enumerate() {
                if !((c >= '0' && c <= '9') || (c >= 'a' && c <= 'f')) {
                    array = None;
                }
                label.push(c);
                if i % 2 == 1 {
                    // labelを16進数として解釈する
                    match u8::from_str_radix(&label, 16) {
                        Ok(v) => match array {
                            Some(ref mut a) => a.push(v),
                            None => (),
                        },
                        Err(_) => array = None,
                    };
                    label = String::new();
                }
            }

            if let Some(ref a) = array {
                self.binary = Some(a.clone());
            } else {
                ui.label(RichText::new("Invalid input.").color(Color32::RED));
            }
            if let Some(ref a) = self.binary {
                egui::ScrollArea::vertical()
                    .enable_scrolling(true)
                    .max_height(screen_rect.height() / 6.0)
                    .show(ui, |ui| {
                        ui.label(format!("{:?}", a));
                    });
            }

            if ui
                .button("Random Generate")
                .on_hover_text("ランダムな数値を生成します")
                .clicked()
            {
                // 0~255の範囲の乱数の配列を生成する
                let mut buf: &mut [u8] = &mut vec![0; self.digit as usize];

                if let Err(_) = getrandom::getrandom(&mut buf) {
                    self.label = "failed to generate random numbers.".into();
                } else {
                    self.label = buf.iter().map(|v| format!("{:02X}", v)).collect::<String>();
                    self.binary = Some(buf.into());
                }
            }

            CollapsingHeader::new("詳細")
                .default_open(true)
                .show(ui, |ui| {
                    ui.spacing_mut().slider_width = 100.0 * self.zoom_level;
                    ui.add(egui::Slider::new(&mut self.digit, 1..=255).text("桁"));
                });

            ui.add(
                egui::Slider::new(&mut self.value, 0.0..=10.0)
                    .trailing_fill(true)
                    .text("value"),
            );

            ui.separator();
            ui.horizontal_wrapped(|ui| {
                ui.style_mut().text_styles = self.default_text_styles.clone().unwrap();
                ui.add(egui::github_link_file!(
                    "https://github.com/oligami-0424/binary-gen-web/blob/main/",
                    "Source code."
                ));
            });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.style_mut().text_styles = self.default_text_styles.clone().unwrap();
                ui.style_mut().spacing.item_spacing.y = 10.0;
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
