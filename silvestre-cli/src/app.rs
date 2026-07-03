//! Application state and logic

use std::path::PathBuf;
use image::ImageReader;
use silvestre_core::{SilvestreImage, ColorSpace};
use crate::filters::{apply_named_filter, is_known_filter, silvestre_to_dynamic, validate_filter};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Screen {
    Main,
    FilterMenu,
    ApplyFilter,
    Pipeline,
    Info,
    Help,
    Processing,
}

/// A single stage in a filter pipeline: a filter name plus the raw parameter
/// string that will be handed to that filter.
#[derive(Debug, Clone, PartialEq)]
pub struct PipelineStep {
    pub filter: String,
    pub params: String,
}

/// Which input box on the pipeline "add step" row currently has focus.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PipelineField {
    /// The name of the filter to add as the next step.
    FilterName,
    /// The parameters for the filter being added.
    Params,
    /// The shared input image path.
    Input,
    /// The shared output image path.
    Output,
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
    // Pipeline screen state.
    pub pipeline_steps: Vec<PipelineStep>,
    pub pipeline_field: PipelineField,
    pub pipeline_filter_input: String,
    pub pipeline_params_input: String,
    pub pipeline_input_file: String,
    pub pipeline_output_file: String,
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
            pipeline_steps: Vec::new(),
            pipeline_field: PipelineField::FilterName,
            pipeline_filter_input: String::new(),
            pipeline_params_input: String::new(),
            pipeline_input_file: String::new(),
            pipeline_output_file: String::new(),
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
        let result_img = apply_named_filter(&silvestre_img, filter_name, params)?;

        // Save result (color space may have changed, e.g. grayscale).
        silvestre_to_dynamic(&result_img)?
            .save(output_path)
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

    // -- Pipeline screen ---------------------------------------------------

    pub fn go_to_pipeline(&mut self) {
        self.current_screen = Screen::Pipeline;
        self.pipeline_field = PipelineField::FilterName;
        self.pipeline_filter_input.clear();
        self.pipeline_params_input.clear();
        self.status_message =
            "Build a pipeline: type a filter, Enter to add. 'r' runs it. 🐾".to_string();
    }

    /// Move focus to the next input box on the pipeline screen.
    pub fn pipeline_next_field(&mut self) {
        self.pipeline_field = match self.pipeline_field {
            PipelineField::FilterName => PipelineField::Params,
            PipelineField::Params => PipelineField::Input,
            PipelineField::Input => PipelineField::Output,
            PipelineField::Output => PipelineField::FilterName,
        };
    }

    /// Move focus to the previous input box on the pipeline screen.
    pub fn pipeline_prev_field(&mut self) {
        self.pipeline_field = match self.pipeline_field {
            PipelineField::FilterName => PipelineField::Output,
            PipelineField::Params => PipelineField::FilterName,
            PipelineField::Input => PipelineField::Params,
            PipelineField::Output => PipelineField::Input,
        };
    }

    pub fn pipeline_input_char(&mut self, c: char) {
        match self.pipeline_field {
            PipelineField::FilterName => self.pipeline_filter_input.push(c),
            PipelineField::Params => self.pipeline_params_input.push(c),
            PipelineField::Input => self.pipeline_input_file.push(c),
            PipelineField::Output => self.pipeline_output_file.push(c),
        }
    }

    pub fn pipeline_input_backspace(&mut self) {
        match self.pipeline_field {
            PipelineField::FilterName => {
                self.pipeline_filter_input.pop();
            }
            PipelineField::Params => {
                self.pipeline_params_input.pop();
            }
            PipelineField::Input => {
                self.pipeline_input_file.pop();
            }
            PipelineField::Output => {
                self.pipeline_output_file.pop();
            }
        }
    }

    /// Add the filter currently typed in the "filter name" box as the next
    /// pipeline step, validating its name and parameters first.
    pub fn pipeline_add_step(&mut self) {
        let filter = self.pipeline_filter_input.trim().to_lowercase();
        let params = self.pipeline_params_input.trim().to_string();

        if filter.is_empty() {
            self.status_message = "Type a filter name before adding a step 🐱".to_string();
            return;
        }
        if !is_known_filter(&filter) {
            self.status_message =
                format!("Unknown filter: {}. Press 'h' for the list 🐾", filter);
            return;
        }
        if let Err(e) = validate_filter(&filter, &params) {
            self.status_message = format!("Invalid params for {}: {} 🐱", filter, e);
            return;
        }

        self.pipeline_steps.push(PipelineStep { filter, params });
        self.pipeline_filter_input.clear();
        self.pipeline_params_input.clear();
        self.pipeline_field = PipelineField::FilterName;
        self.status_message = format!(
            "Added step {}. Add more, or press 'r' to run. 🐾",
            self.pipeline_steps.len()
        );
    }

    /// Remove the most recently added pipeline step.
    pub fn pipeline_remove_last(&mut self) {
        if let Some(removed) = self.pipeline_steps.pop() {
            self.status_message = format!("Removed step: {} 🐾", removed.filter);
        } else {
            self.status_message = "No steps to remove 🐱".to_string();
        }
    }

    /// Clear every step from the pipeline.
    pub fn pipeline_clear(&mut self) {
        self.pipeline_steps.clear();
        self.status_message = "Pipeline cleared 🐾".to_string();
    }

    /// Validate and run the whole pipeline, applying each step in order.
    pub fn pipeline_run_action(&mut self) {
        if self.pipeline_steps.is_empty() {
            self.status_message = "Add at least one filter before running 🐱".to_string();
            return;
        }
        if self.pipeline_input_file.trim().is_empty()
            || self.pipeline_output_file.trim().is_empty()
        {
            self.status_message = "Please specify input and output files 🐱".to_string();
            return;
        }

        self.current_screen = Screen::Processing;
        self.processing = true;
        self.status_message = "Running pipeline... Silvestre is concentrating...".to_string();

        let input_path = self.pipeline_input_file.trim().to_string();
        let output_path = self.pipeline_output_file.trim().to_string();
        let steps = self.pipeline_steps.clone();

        let result = run_pipeline(&input_path, &output_path, &steps);

        self.processing = false;
        self.status_message = match result {
            Ok(msg) => msg,
            Err(e) => format!("Error: {} 🐱", e),
        };
    }
}

/// Validate every step up front, then load the image, apply each step in
/// sequence (feeding one step's output into the next), and save the result.
///
/// On failure the returned error names the 1-based step and filter that
/// failed, so the user knows exactly where the pipeline broke.
pub fn run_pipeline(
    input_path: &str,
    output_path: &str,
    steps: &[PipelineStep],
) -> Result<String, String> {
    if steps.is_empty() {
        return Err("Pipeline has no steps".to_string());
    }

    // Validate the full pipeline before doing any image work, so a typo in a
    // later step fails fast instead of after processing earlier ones.
    for (idx, step) in steps.iter().enumerate() {
        validate_filter(&step.filter, &step.params)
            .map_err(|e| format!("Step {} ({}): {}", idx + 1, step.filter, e))?;
    }

    let input_file = PathBuf::from(input_path);
    if !input_file.exists() {
        return Err("Input file not found!".to_string());
    }

    let reader =
        ImageReader::open(&input_file).map_err(|e| format!("Failed to read image: {}", e))?;
    let dynamic_image = reader
        .decode()
        .map_err(|e| format!("Failed to decode image: {}", e))?;
    let rgba_image = dynamic_image.to_rgba8();
    let (width, height) = rgba_image.dimensions();
    let pixels = rgba_image.into_raw();

    let mut current = SilvestreImage::new(pixels, width, height, ColorSpace::Rgba)
        .map_err(|e| format!("Image error: {}", e))?;

    // Apply each step, passing the output of one as the input of the next.
    for (idx, step) in steps.iter().enumerate() {
        current = apply_named_filter(&current, &step.filter, &step.params)
            .map_err(|e| format!("Step {} ({}): {}", idx + 1, step.filter, e))?;
    }

    if current.width() == 0 || current.height() == 0 {
        return Err("Pipeline produced an empty image".to_string());
    }

    silvestre_to_dynamic(&current)?
        .save(output_path)
        .map_err(|e| format!("Failed to save image: {}", e))?;

    Ok(format!(
        "Pipeline of {} step(s) applied successfully! 🎉 Saved to {}",
        steps.len(),
        output_path
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn step(filter: &str, params: &str) -> PipelineStep {
        PipelineStep {
            filter: filter.to_string(),
            params: params.to_string(),
        }
    }

    /// Write a small solid-color PNG to a unique path in the temp dir and
    /// return that path. `tag` keeps parallel tests from colliding.
    fn write_test_png(tag: &str, w: u32, h: u32) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("silvestre_pipeline_{}_{}.png", std::process::id(), tag));
        let buffer = image::RgbaImage::from_pixel(w, h, image::Rgba([100, 150, 200, 255]));
        image::DynamicImage::ImageRgba8(buffer).save(&path).unwrap();
        path
    }

    fn out_path(tag: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("silvestre_pipeline_out_{}_{}.png", std::process::id(), tag));
        path
    }

    #[test]
    fn run_pipeline_applies_steps_in_order() {
        let input = write_test_png("order_in", 40, 40);
        let output = out_path("order");
        let steps = vec![step("grayscale", ""), step("resize", "20,10")];

        let msg = run_pipeline(
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            &steps,
        )
        .expect("pipeline should succeed");

        assert!(msg.contains("2 step(s)"));

        // Resize was the last step, so the output must be 20x10.
        let saved = image::open(&output).unwrap();
        assert_eq!(saved.width(), 20);
        assert_eq!(saved.height(), 10);

        let _ = std::fs::remove_file(&input);
        let _ = std::fs::remove_file(&output);
    }

    #[test]
    fn run_pipeline_ending_in_grayscale_saves() {
        // Grayscale changes the color space to 1 channel; saving must honor
        // that instead of assuming an RGBA buffer.
        let input = write_test_png("gray_in", 16, 16);
        let output = out_path("gray");
        let steps = vec![step("invert", ""), step("grayscale", "")];

        run_pipeline(input.to_str().unwrap(), output.to_str().unwrap(), &steps)
            .expect("grayscale-terminated pipeline should save");

        let saved = image::open(&output).unwrap();
        assert_eq!(saved.width(), 16);
        assert_eq!(saved.height(), 16);

        let _ = std::fs::remove_file(&input);
        let _ = std::fs::remove_file(&output);
    }

    #[test]
    fn run_pipeline_reports_failing_step_number() {
        let input = write_test_png("badstep_in", 20, 20);
        let output = out_path("badstep");
        // Step 2 crops outside the image bounds and should fail.
        let steps = vec![
            step("grayscale", ""),
            step("crop", "50,50,10,10"),
            step("invert", ""),
        ];

        let err = run_pipeline(
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            &steps,
        )
        .unwrap_err();

        assert!(err.contains("Step 2"), "error was: {}", err);
        assert!(err.contains("crop"), "error was: {}", err);
        // The output file must not have been created on failure.
        assert!(!output.exists());

        let _ = std::fs::remove_file(&input);
    }

    #[test]
    fn run_pipeline_validates_before_processing() {
        let input = write_test_png("validate_in", 20, 20);
        let output = out_path("validate");
        // Invalid params in step 2 should fail during up-front validation,
        // before the image is even loaded.
        let steps = vec![step("grayscale", ""), step("brightness", "notanumber")];

        let err = run_pipeline(
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            &steps,
        )
        .unwrap_err();

        assert!(err.contains("Step 2"), "error was: {}", err);
        assert!(!output.exists());

        let _ = std::fs::remove_file(&input);
    }

    #[test]
    fn run_pipeline_rejects_empty_pipeline() {
        let err = run_pipeline("whatever.png", "out.png", &[]).unwrap_err();
        assert!(err.contains("no steps"));
    }

    #[test]
    fn run_pipeline_rejects_missing_input() {
        let steps = vec![step("grayscale", "")];
        let err = run_pipeline("/no/such/file.png", "out.png", &steps).unwrap_err();
        assert!(err.contains("not found"));
    }

    #[test]
    fn add_step_appends_valid_filter() {
        let mut app = App::new();
        app.go_to_pipeline();
        app.pipeline_filter_input = "grayscale".to_string();
        app.pipeline_add_step();

        assert_eq!(app.pipeline_steps.len(), 1);
        assert_eq!(app.pipeline_steps[0].filter, "grayscale");
        // Inputs are cleared after a successful add.
        assert!(app.pipeline_filter_input.is_empty());
    }

    #[test]
    fn add_step_lowercases_and_trims_filter() {
        let mut app = App::new();
        app.pipeline_filter_input = "  GrayScale  ".to_string();
        app.pipeline_add_step();
        assert_eq!(app.pipeline_steps[0].filter, "grayscale");
    }

    #[test]
    fn add_step_rejects_unknown_filter() {
        let mut app = App::new();
        app.pipeline_filter_input = "bogus".to_string();
        app.pipeline_add_step();
        assert!(app.pipeline_steps.is_empty());
        assert!(app.status_message.contains("Unknown filter"));
    }

    #[test]
    fn add_step_rejects_invalid_params() {
        let mut app = App::new();
        app.pipeline_filter_input = "brightness".to_string();
        app.pipeline_params_input = "abc".to_string();
        app.pipeline_add_step();
        assert!(app.pipeline_steps.is_empty());
        assert!(app.status_message.contains("Invalid params"));
    }

    #[test]
    fn add_step_rejects_empty_filter() {
        let mut app = App::new();
        app.pipeline_filter_input = "   ".to_string();
        app.pipeline_add_step();
        assert!(app.pipeline_steps.is_empty());
    }

    #[test]
    fn remove_last_and_clear_work() {
        let mut app = App::new();
        app.pipeline_steps = vec![step("grayscale", ""), step("invert", "")];

        app.pipeline_remove_last();
        assert_eq!(app.pipeline_steps.len(), 1);

        app.pipeline_clear();
        assert!(app.pipeline_steps.is_empty());

        // Removing from an empty pipeline is a no-op with a message.
        app.pipeline_remove_last();
        assert!(app.status_message.contains("No steps"));
    }

    #[test]
    fn field_navigation_cycles() {
        let mut app = App::new();
        assert_eq!(app.pipeline_field, PipelineField::FilterName);
        app.pipeline_next_field();
        assert_eq!(app.pipeline_field, PipelineField::Params);
        app.pipeline_prev_field();
        assert_eq!(app.pipeline_field, PipelineField::FilterName);
        app.pipeline_prev_field();
        assert_eq!(app.pipeline_field, PipelineField::Output);
    }

    #[test]
    fn run_action_requires_steps_and_files() {
        let mut app = App::new();
        // No steps yet.
        app.pipeline_run_action();
        assert!(app.status_message.contains("at least one filter"));

        // Steps present but no files.
        app.pipeline_steps = vec![step("grayscale", "")];
        app.pipeline_run_action();
        assert!(app.status_message.contains("input and output"));
    }
}
