use crate::doom_fire::DoomFire;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App {
    #[serde(skip)]
    fire: DoomFire,

    #[serde(skip)]
    show_ui: bool,

    #[serde(skip)]
    texture: Option<egui::TextureHandle>,

    fire_width: usize,
    fire_height: usize,
}

impl Default for App {
    fn default() -> Self {
        let fire_width = 320;
        let fire_height = 168;
        Self {
            fire: DoomFire::new(fire_width, fire_height),
            fire_width,
            fire_height,
            show_ui: true,
            texture: None,
        }
    }
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }
}

impl eframe::App for App {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.fire.do_fire();
        let fire_width = self.fire.width;
        let fire_height = self.fire.height;
        let rgba = self.fire.get_rgba_buffer();

        let texture = self.texture.get_or_insert_with(|| {
            ctx.load_texture(
                "fire",
                egui::ColorImage::from_rgba_unmultiplied([fire_width, fire_height], rgba),
                egui::TextureOptions::NEAREST,
            )
        });

        if texture.size() != [fire_width, fire_height] {
            *texture = ctx.load_texture(
                "fire",
                egui::ColorImage::from_rgba_unmultiplied([fire_width, fire_height], rgba),
                egui::TextureOptions::NEAREST,
            );
        } else {
            texture.set_partial(
                [0, 0],
                egui::ColorImage::from_rgba_unmultiplied([fire_width, fire_height], rgba),
                egui::TextureOptions::NEAREST,
            );
        }

        if ctx.input(|i| i.key_pressed(egui::Key::H)) {
            self.show_ui = !self.show_ui;
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                if let Some(tex) = &self.texture {
                    ui.image((tex.id(), ui.available_size()));
                }
            });

        if self.show_ui {
            egui::Window::new("Doom Fire Options")
                .default_open(true)
                .show(ctx, |ui| {
                    ui.label("Press 'H' to hide this window");

                    let mut changed = false;
                    changed |= ui
                        .add(egui::Slider::new(&mut self.fire_width, 64..=640).text("Width"))
                        .changed();
                    changed |= ui
                        .add(egui::Slider::new(&mut self.fire_height, 32..=360).text("Height"))
                        .changed();

                    if changed {
                        self.fire.resize(self.fire_width, self.fire_height);
                        self.texture = None;
                    }
                });
        }

        ctx.request_repaint();
    }
}
