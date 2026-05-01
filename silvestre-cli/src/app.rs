//! Application state and logic

use std::path::PathBuf;
use image::ImageReader;
use silvestre_core::{SilvestreImage, ColorSpace};
use silvestre_core::filters::Filter;
use silvestre_core::effects::{BrightnessFilter, ContrastFilter, GrayscaleFilter, InvertFilter};
use silvestre_core::transform::{CropFilter, MirrorFilter, MirrorMode, ResizeFilter, RotateFilter, Interpolation};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Screen {
    Main,
    FilterMenu,
    ApplyFilter,
    Info,
    Help,
    Processing,
}

pub struct FilterInfo {
    pub name: &'static str,
    pub category: &'static str,
    pub description: &'static str,
}

pub struct App {
    pub current_screen: Screen,
    pub filters: Vec<FilterInfo>,
    pub selected_filter: usize,
    pub selected_field: usize,
    pub input_file: String,
    pub output_file: String,
    pub filter_params: String,
    pub info_input: String,
    pub status_message: String,
    pub processing: bool,
}

impl App {
    pub fn new() -> Self {
        let filters = vec![
            FilterInfo {
                name: "brightness",
                category: "Effects",
                description: "Adjust brightness (delta: -255 to 255)",
            },
            FilterInfo {
                name: "contrast",
                category: "Effects",
                description: "Adjust contrast (factor: 0.0 to ∞)",
            },
            FilterInfo {
                name: "grayscale",
                category: "Effects",
                description: "Convert to grayscale",
            },
            FilterInfo {
                name: "invert",
                category: "Effects",
                description: "Invert colors",
            },
            FilterInfo {
                name: "crop",
                category: "Transforms",
                description: "Crop image (x, y, width, height)",
            },
            FilterInfo {
                name: "mirror",
                category: "Transforms",
                description: "Mirror/flip image (h|v|both)",
            },
            FilterInfo {
                name: "resize",
                category: "Transforms",
                description: "Resize image (width, height)",
            },
            FilterInfo {
                name: "rotate",
                category: "Transforms",
                description: "Rotate image (angle in degrees)",
            },
        ];

        Self {
            current_screen: Screen::Main,
            filters,
            selected_filter: 0,
            selected_field: 0,
            input_file: String::new(),
            output_file: String::new(),
            filter_params: String::new(),
            info_input: String::new(),
            status_message: "🐱 Welcome to Silvestre (named after my magnificent cat!)".to_string(),
            processing: false,
        }
    }

    // Navigation
    pub fn go_to_main(&mut self) {
        self.current_screen = Screen::Main;
        self.status_message = "Back to main menu. 🐾".to_string();
    }

    pub fn go_to_filter_menu(&mut self) {
        self.current_screen = Screen::FilterMenu;
        self.selected_filter = 0;
        self.status_message = "Select a filter (use ↑↓ arrows)".to_string();
    }

    pub fn go_to_apply_filter(&mut self) {
        self.current_screen = Screen::ApplyFilter;
        self.input_file.clear();
        self.output_file.clear();
        self.filter_params.clear();
        self.selected_field = 0;
        self.status_message = format!(
            "Applying filter: {} (as sleek as Silvestre's whiskers)",
            self.filters[self.selected_filter].name
        );
    }

    pub fn go_to_info(&mut self) {
        self.current_screen = Screen::Info;
        self.info_input.clear();
        self.status_message = "Enter image path to inspect (Silvestre is watching...)".to_string();
    }

    pub fn go_to_help(&mut self) {
        self.current_screen = Screen::Help;
        self.status_message = "Silvestre's wisdom awaits...".to_string();
    }

    // Filter menu navigation
    pub fn select_previous_filter(&mut self) {
        if self.selected_filter > 0 {
            self.selected_filter -= 1;
        } else {
            self.selected_filter = self.filters.len() - 1;
        }
    }

    pub fn select_next_filter(&mut self) {
        self.selected_filter = (self.selected_filter + 1) % self.filters.len();
    }

    // Apply filter navigation
    pub fn next_field(&mut self) {
        self.selected_field = (self.selected_field + 1) % 4;
    }

    pub fn prev_field(&mut self) {
        if self.selected_field == 0 {
            self.selected_field = 3;
        } else {
            self.selected_field -= 1;
        }
    }

    pub fn input_char(&mut self, c: char) {
        match self.selected_field {
            0 => self.input_file.push(c),
            1 => self.output_file.push(c),
            2 => self.filter_params.push(c),
            _ => {}
        }
    }

    pub fn input_backspace(&mut self) {
        match self.selected_field {
            0 => {
                self.input_file.pop();
            }
            1 => {
                self.output_file.pop();
            }
            2 => {
                self.filter_params.pop();
            }
            _ => {}
        }
    }

    pub fn is_apply_button_focused(&self) -> bool {
        self.selected_field == 3
    }

    pub fn apply_filter_action(&mut self) {
        if self.input_file.is_empty() || self.output_file.is_empty() {
            self.status_message = "Please specify input and output files 🐱".to_string();
            return;
        }

        self.current_screen = Screen::Processing;
        self.processing = true;
        self.status_message = "Processing image... Silvestre is concentrating...".to_string();

        let filter_name = self.filters[self.selected_filter].name;
        let input_path = self.input_file.clone();
        let output_path = self.output_file.clone();
        let params = self.filter_params.clone();

        let result = self.apply_filter_impl(&input_path, &output_path, filter_name, &params);

        self.processing = false;
        self.status_message = match result {
            Ok(msg) => msg,
            Err(e) => format!("Error: {} 🐱", e),
        };
    }

    fn apply_filter_impl(&self, input_path: &str, output_path: &str, filter_name: &str, params: &str) -> Result<String, String> {
        let input_file = PathBuf::from(input_path);
        if !input_file.exists() {
            return Err("Input file not found!".to_string());
        }

        // Load image
        let reader = ImageReader::open(&input_file)
            .map_err(|e| format!("Failed to read image: {}", e))?;
        let dynamic_image = reader.decode()
            .map_err(|e| format!("Failed to decode image: {}", e))?;
        let rgba_image = dynamic_image.to_rgba8();
        let (width, height) = rgba_image.dimensions();
        let pixels = rgba_image.into_raw();

        // Create SilvestreImage
        let silvestre_img = SilvestreImage::new(pixels, width, height, ColorSpace::Rgba)
            .map_err(|e| format!("Image error: {}", e))?;

        // Apply filter based on selection
        let result_img = match filter_name {
            "brightness" => {
                let delta = params.trim().parse::<i32>()
                    .map_err(|_| "Brightness requires a number (-255 to 255)".to_string())?;
                let filter = BrightnessFilter::new(delta);
                filter.apply(&silvestre_img)
                    .map_err(|e| format!("Brightness filter error: {}", e))?
            },
            "contrast" => {
                let factor = params.trim().parse::<f32>()
                    .map_err(|_| "Contrast requires a decimal number".to_string())?;
                let filter = ContrastFilter::new(factor)
                    .map_err(|e| format!("Contrast filter error: {}", e))?;
                filter.apply(&silvestre_img)
                    .map_err(|e| format!("Contrast filter error: {}", e))?
            },
            "grayscale" => {
                let filter = GrayscaleFilter;
                filter.apply(&silvestre_img)
                    .map_err(|e| format!("Grayscale filter error: {}", e))?
            },
            "invert" => {
                let filter = InvertFilter;
                filter.apply(&silvestre_img)
                    .map_err(|e| format!("Invert filter error: {}", e))?
            },
            "crop" => {
                let parts: Vec<&str> = params.trim().split(',').collect();
                if parts.len() != 4 {
                    return Err("Crop requires 4 params: x, y, width, height".to_string());
                }
                let x = parts[0].trim().parse::<u32>()
                    .map_err(|_| "Invalid crop x coordinate".to_string())?;
                let y = parts[1].trim().parse::<u32>()
                    .map_err(|_| "Invalid crop y coordinate".to_string())?;
                let w = parts[2].trim().parse::<u32>()
                    .map_err(|_| "Invalid crop width".to_string())?;
                let h = parts[3].trim().parse::<u32>()
                    .map_err(|_| "Invalid crop height".to_string())?;
                let filter = CropFilter::new(x, y, w, h);
                filter.apply(&silvestre_img)
                    .map_err(|e| format!("Crop filter error: {}", e))?
            },
            "mirror" => {
                let mode = params.trim().to_lowercase();
                let mirror_mode = match mode.as_str() {
                    "h" | "horizontal" => MirrorMode::Horizontal,
                    "v" | "vertical" => MirrorMode::Vertical,
                    "both" => MirrorMode::Both,
                    _ => return Err("Mirror mode should be: h, v, or both".to_string()),
                };
                let filter = MirrorFilter::new(mirror_mode);
                filter.apply(&silvestre_img)
                    .map_err(|e| format!("Mirror filter error: {}", e))?
            },
            "resize" => {
                let parts: Vec<&str> = params.trim().split(',').collect();
                if parts.len() != 2 {
                    return Err("Resize requires 2 params: width, height".to_string());
                }
                let w = parts[0].trim().parse::<u32>()
                    .map_err(|_| "Invalid resize width".to_string())?;
                let h = parts[1].trim().parse::<u32>()
                    .map_err(|_| "Invalid resize height".to_string())?;
                let filter = ResizeFilter::new(w, h, Interpolation::Bilinear);
                filter.apply(&silvestre_img)
                    .map_err(|e| format!("Resize filter error: {}", e))?
            },
            "rotate" => {
                let angle = params.trim().parse::<f64>()
                    .map_err(|_| "Rotate requires an angle in degrees".to_string())?;
                let filter = RotateFilter::new(angle, 0, [0, 0, 0]);
                filter.apply(&silvestre_img)
                    .map_err(|e| format!("Rotate filter error: {}", e))?
            },
            _ => return Err("Unknown filter".to_string()),
        };

        // Save result
        let pixels = result_img.pixels().to_vec();
        let img_buffer = image::ImageBuffer::<image::Rgba<u8>, Vec<u8>>::from_raw(
            result_img.width(),
            result_img.height(),
            pixels,
        ).ok_or_else(|| "Failed to create image buffer".to_string())?;

        image::DynamicImage::ImageRgba8(img_buffer)
            .save(&output_path)
            .map_err(|e| format!("Failed to save image: {}", e))?;

        Ok(format!("Filter applied successfully! 🎉 Saved to {}", output_path))
    }

    pub fn is_processing_done(&self) -> bool {
        self.current_screen == Screen::Processing && !self.processing
    }

    // Info screen
    pub fn info_input_char(&mut self, c: char) {
        self.info_input.push(c);
    }

    pub fn info_input_backspace(&mut self) {
        self.info_input.pop();
    }

    pub fn load_image_info(&mut self) {
        if self.info_input.is_empty() {
            self.status_message = "Please enter a file path 🐱".to_string();
            return;
        }

        let path = PathBuf::from(&self.info_input);
        if !path.exists() {
            self.status_message = "File not found! Silvestre couldn't find it either 🔍".to_string();
            return;
        }

        match ImageReader::open(&path) {
            Ok(reader) => {
                match reader.decode() {
                    Ok(image) => {
                        let width = image.width();
                        let height = image.height();
                        let color_type = format!("{:?}", image.color());
                        self.status_message = format!(
                            "📷 {}x{} {} (Silvestre approves ✓)",
                            width, height, color_type
                        );
                    }
                    Err(e) => {
                        self.status_message = format!("Decode error: {} 🐱", e);
                    }
                }
            }
            Err(e) => {
                self.status_message = format!("Read error: {} 🐱", e);
            }
        }
    }
}
