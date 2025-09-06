use eframe::egui;
use egui_extras::RetainedImage;
use image::GenericImageView;
use rodio::{Decoder, OutputStream, Sink};
use std::{env, fs::File, io::BufReader, path::Path};

fn main() -> eframe::Result<()> {
    let args: Vec<String> = env::args().collect();
    let path = args.get(1).cloned().expect("Please provide a file path!");

    let ext = Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let options = eframe::NativeOptions::default();

    let audio_exts = ["mp3", "wav", "ogg", "flac"];
    let image_exts = ["png", "jpg", "jpeg", "bmp", "tiff", "gif", "ico"];

    if audio_exts.contains(&ext.as_str()) {
        eframe::run_native(
            "Audio Player",
            options,
            Box::new(|_cc| Box::new(AudioApp::new(path))),
        )
    } else if image_exts.contains(&ext.as_str()) {
        eframe::run_native(
            "Image Viewer",
            options,
            Box::new(|_cc| Box::new(ImageApp::new(Some(path)))),
        )
    } else {
        panic!("Unsupported file format: {}", ext);
    }
}

struct AudioApp {
    sink: Option<Sink>,
    _stream: Option<OutputStream>,
    file_path: String,
    is_playing: bool,
}

impl AudioApp {
    fn new(path: String) -> Self {
        Self {
            sink: None,
            _stream: None,
            file_path: path,
            is_playing: false,
        }
    }

    fn play(&mut self) {
        if self.is_playing {
            return;
        }
        let (_stream, handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&handle).unwrap();
        let file = File::open(&self.file_path).unwrap();
        let source = Decoder::new(BufReader::new(file)).unwrap();
        sink.append(source);
        self._stream = Some(_stream);
        self.sink = Some(sink);
        self.is_playing = true;
    }

    fn pause(&mut self) {
        if let Some(sink) = &self.sink {
            sink.pause();
        }
    }

    fn resume(&mut self) {
        if let Some(sink) = &self.sink {
            sink.play();
        }
    }

    fn stop(&mut self) {
        if let Some(sink) = &self.sink {
            sink.stop();
        }
        self.sink = None;
        self._stream = None;
        self.is_playing = false;
    }
}

impl eframe::App for AudioApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Audio Player");
                ui.add_space(20.0);

                ui.horizontal_centered(|ui| {
                    if ui.button("PLAY").clicked() {
                        self.play();
                    }
                    if ui.button("PAUSE").clicked() {
                        self.pause();
                    }
                    if ui.button("RESUME").clicked() {
                        self.resume();
                    }
                    if ui.button("STOP").clicked() {
                        self.stop();
                    }
                });
            });
        });
    }
}

struct ImageApp {
    retained_image: Option<RetainedImage>,
}

impl ImageApp {
    fn new(path: Option<String>) -> Self {
        let loaded_image = path.as_deref().map(|p| load_image(p).ok()).flatten();
        Self {
            retained_image: loaded_image,
        }
    }
}

fn load_image(path: &str) -> Result<RetainedImage, String> {
    let img = image::open(path).map_err(|e| format!("Could not open image: {}", e))?;
    let dims = img.dimensions();
    let rgba = img.to_rgba8();
    let pixels: Vec<egui::Color32> = rgba
        .pixels()
        .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
        .collect();
    let color_image = egui::ColorImage {
        size: [dims.0 as usize, dims.1 as usize],
        pixels,
    };
    Ok(RetainedImage::from_color_image("image", color_image))
}

impl eframe::App for ImageApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui_extras::install_image_loaders(ctx);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Image Viewer");
                if let Some(img) = &self.retained_image {
                    let size = ui.available_size();
                    img.show_max_size(ui, size);
                }
            });
        });
    }
}
