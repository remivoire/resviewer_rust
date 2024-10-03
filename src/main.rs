use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use byteorder::{LittleEndian, ReadBytesExt};
use eframe::egui;
use rfd::FileDialog;
use egui::FontDefinitions;

const MAGIC_ILFF: u32 = 0x46464C49; // 'ILFF'
const RES_TYPE_IRES: u32 = 0x53455249; // 'IRES'
const CHUNK_TYPE_NAME: u32 = 0x454D414E; // 'NAME'
const CHUNK_TYPE_BODY: u32 = 0x59444F42; // 'BODY'

struct ImageResource {
    name: Option<String>,
    width: u16,
    height: u16,
    data: Vec<u8>,
}

fn read_ilff_file(filename: &str, debug_log: &mut Vec<String>) -> io::Result<Vec<ImageResource>> {
    debug_log.push(format!("Opening file: {}", filename));
    let mut file = File::open(filename)?;

    let magic = file.read_u32::<LittleEndian>()?;
    debug_log.push(format!("Read magic number: 0x{:08X}", magic));
    if magic != MAGIC_ILFF {
        debug_log.push("Invalid magic number!".to_string());
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid magic number"));
    }

    let _filesize = file.read_u32::<LittleEndian>()?;
    let _alignment = file.read_u32::<LittleEndian>()?;
    let _reserve = file.read_u32::<LittleEndian>()?;
    let res_type = file.read_u32::<LittleEndian>()?;
    debug_log.push(format!("Resource type: 0x{:08X}", res_type));
    if res_type != RES_TYPE_IRES {
        debug_log.push("Invalid resource type!".to_string());
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid resource type"));
    }

    let mut images = Vec::new();
    let mut current_name: Option<String> = None;

    while let Ok(chunk_type) = file.read_u32::<LittleEndian>() {
        let buffer_size = file.read_u32::<LittleEndian>()?;
        let alignment = file.read_u32::<LittleEndian>()?;
        let _chunk_size = file.read_u32::<LittleEndian>()?;
        debug_log.push(format!("Reading chunk type: 0x{:08X} with buffer size: {}", chunk_type, buffer_size));

        let chunk_start = file.seek(SeekFrom::Current(0))?;

        match chunk_type {
            CHUNK_TYPE_NAME => {
                let mut name_bytes = vec![0u8; buffer_size as usize];
                file.read_exact(&mut name_bytes)?;
                let name = String::from_utf8_lossy(&name_bytes)
                    .trim_end_matches('\0')
                    .to_string();
                debug_log.push(format!("Found NAME chunk: {}", name));
                current_name = Some(name);
            }
            CHUNK_TYPE_BODY => {
                debug_log.push("Found BODY chunk.".to_string());
                let _body_type = file.read_u32::<LittleEndian>()?;
                let _unk1 = file.read_u32::<LittleEndian>()?;
                let _unk2 = file.read_u32::<LittleEndian>()?;
                let _unk3 = file.read_u32::<LittleEndian>()?;
                let _unk4 = file.read_u32::<LittleEndian>()?;
                let _unk5 = file.read_u16::<LittleEndian>()?;
                let width_1 = file.read_u16::<LittleEndian>()?;
                let height_1 = file.read_u16::<LittleEndian>()?;
                let _width_2 = file.read_u16::<LittleEndian>()?;
                let _height_2 = file.read_u16::<LittleEndian>()?;
                let _unk6 = file.read_u16::<LittleEndian>()?;

                let subheader_size = 32;

                if buffer_size < subheader_size {
                    debug_log.push("Invalid buffer size for BODY chunk.".to_string());
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid buffer size"));
                }

                let image_data_size = buffer_size - subheader_size;

                let mut image_data = vec![0u8; image_data_size as usize];
                file.read_exact(&mut image_data)?;

                let expected_size = (width_1 as usize) * (height_1 as usize) * 4;
                if image_data.len() < expected_size {
                    debug_log.push("Truncating image data due to unexpected size.".to_string());
                    continue;
                } else if image_data.len() > expected_size {
                    image_data.truncate(expected_size);
                }

                let image = ImageResource {
                    name: current_name.clone(),
                    width: width_1,
                    height: height_1,
                    data: image_data,
                };

                debug_log.push(format!(
                    "Loaded image: {:?} | Resolution: {}x{} | Size: {} bytes",
                    image.name, image.width, image.height, image.data.len()
                ));
                images.push(image);
            }
            _ => {
                debug_log.push(format!("Skipping unknown chunk type: 0x{:08X}", chunk_type));
                file.seek(SeekFrom::Start(chunk_start + buffer_size as u64))?;
            }
        }

        let current_pos = file.seek(SeekFrom::Current(0))?;
        let padding = (alignment as u64 - (current_pos % alignment as u64)) % alignment as u64;
        file.seek(SeekFrom::Current(padding as i64))?;
    }

    Ok(images)
}

struct MyApp {
    images: Vec<ImageResource>,
    selected_index: Option<usize>,
    textures: Vec<Option<egui::TextureHandle>>,
    file_path: Option<String>,
    error_message: Option<String>,
    show_debug_console: bool,
    debug_log: Vec<String>,
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            "Inter".to_owned(),
            egui::FontData::from_static(include_bytes!("fonts/Inter-Regular.ttf")),
        );
        fonts.families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "Inter".to_owned());
        fonts.families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .push("Inter".to_owned());
        cc.egui_ctx.set_fonts(fonts);

        Self {
            images: Vec::new(),
            selected_index: None,
            textures: Vec::new(),
            file_path: None,
            error_message: None,
            show_debug_console: false,
            debug_log: Vec::new(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        if let Some(path) = FileDialog::new()
                            .add_filter("Resource Files", &["res"])
                            .set_directory(".")
                            .pick_file()
                        {
                            let path_str = path.to_string_lossy().to_string();
                            match read_ilff_file(&path_str, &mut self.debug_log) {
                                Ok(images) => {
                                    self.images = images;
                                    self.file_path = Some(path_str);
                                    self.error_message = None;
                                    self.debug_log.push("File successfully loaded.".to_string());
                                }
                                Err(e) => {
                                    self.error_message = Some(format!("Failed to read file: {}", e));
                                    self.debug_log.push(format!("Failed to read file: {}", e));
                                }
                            }
                        }
                        ui.close_menu();
                    }
                });
                ui.menu_button("Debug", |ui| {
                    if ui.checkbox(&mut self.show_debug_console, "Debug Console").clicked() {
                        ui.close_menu();
                    }
                });
            });
        });

        egui::SidePanel::left("image_list").resizable(true).show(ctx, |ui| {
            ui.heading("Images");
            for (i, image) in self.images.iter().enumerate() {
                let name = image.name.clone().unwrap_or_else(|| format!("Image {}", i));
                if ui.selectable_label(self.selected_index == Some(i), &name).clicked() {
                    self.selected_index = Some(i);
                }
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(index) = self.selected_index {
                let image = &self.images[index];
                if self.textures.len() <= index {
                    self.textures.resize(index + 1, None);
                }
                if self.textures[index].is_none() {
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(
                        [image.width as usize, image.height as usize],
                        &image.data,
                    );
                    let texture = ctx.load_texture(
                        format!("image_{}", index),
                        color_image,
                        egui::TextureOptions::default(),
                    );
                    self.textures[index] = Some(texture);
                }
                ui.label(format!(
                    "Resolution: {}x{} | Size: {} bytes",
                    image.width, image.height, image.data.len()
                ));
                if let Some(texture) = &self.textures[index] {
                    ui.add(egui::Image::new((texture.id(), texture.size_vec2())));
                }
            } else {
                ui.label("Select an image from the list.");
            }
        });

        if self.show_debug_console {
            egui::Window::new("Debug Console")
                .resizable(true)
                .scroll([true, true])  // scropllability
                .default_size([500.0, 300.0])
                .open(&mut self.show_debug_console)
                .show(ctx, |ui| {
                    ui.label("Debug Output:");
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for log in &self.debug_log {
                            ui.monospace(log);
                        }
                    });
                    if let Some(error) = &self.error_message {
                        ui.monospace(format!("Error: {}", error));
                    }
                });
        }
    }
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "IGI TEX Viewer",
        native_options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
    .unwrap();
}