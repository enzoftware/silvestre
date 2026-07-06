//! Application state and logic

use std::path::PathBuf;
use image::ImageReader;
use silvestre_core::{SilvestreImage, ColorSpace};
use crate::filters::{apply_named_filter, silvestre_to_dynamic, validate_filter, KNOWN_FILTERS};

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

/// One row in the pipeline's checkbox list: a known filter, whether it is
/// enabled (checked), and the params typed for it. Rows are kept in a fixed
/// order and enabled ones run top→bottom.
#[derive(Debug, Clone, PartialEq)]
pub struct PipelineFilter {
    pub name: String,
    /// Human-readable hint for the params this filter expects.
    pub hint: String,
    /// Whether this filter is checked and will run.
    pub enabled: bool,
    /// The raw params string for this filter (only used when enabled).
    pub params: String,
}

/// Which area of the pipeline screen currently has focus.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PipelineField {
    /// The checkbox list of filters (navigate with ↑↓, Space toggles, typing
    /// edits the highlighted enabled filter's params).
    Filters,
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
    /// The checkbox list of filters, in fixed run order.
    pub pipeline_filters: Vec<PipelineFilter>,
    /// Which filter row is highlighted in the list.
    pub pipeline_selected: usize,
    pub pipeline_field: PipelineField,
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

        // The pipeline checkbox list mirrors the canonical filter list, in the
        // same fixed order that enabled filters will run in.
        let pipeline_filters = KNOWN_FILTERS
            .iter()
            .map(|(name, hint)| PipelineFilter {
                name: name.to_string(),
                hint: hint.to_string(),
                enabled: false,
                params: String::new(),
            })
            .collect();

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
            pipeline_filters,
            pipeline_selected: 0,
            pipeline_field: PipelineField::Filters,
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
        self.pipeline_field = PipelineField::Filters;
        self.pipeline_selected = 0;
        self.status_message =
            "Check filters with Space, type params, Ctrl+R to run. 🐾".to_string();
    }

    /// Move focus to the next area on the pipeline screen (list → input → output).
    pub fn pipeline_next_field(&mut self) {
        self.pipeline_field = match self.pipeline_field {
            PipelineField::Filters => PipelineField::Input,
            PipelineField::Input => PipelineField::Output,
            PipelineField::Output => PipelineField::Filters,
        };
    }

    /// Move focus to the previous area on the pipeline screen.
    pub fn pipeline_prev_field(&mut self) {
        self.pipeline_field = match self.pipeline_field {
            PipelineField::Filters => PipelineField::Output,
            PipelineField::Input => PipelineField::Filters,
            PipelineField::Output => PipelineField::Input,
        };
    }

    /// Highlight the previous filter row in the checkbox list (wraps around).
    pub fn pipeline_select_previous(&mut self) {
        if self.pipeline_filters.is_empty() {
            return;
        }
        if self.pipeline_selected > 0 {
            self.pipeline_selected -= 1;
        } else {
            self.pipeline_selected = self.pipeline_filters.len() - 1;
        }
    }

    /// Highlight the next filter row in the checkbox list (wraps around).
    pub fn pipeline_select_next(&mut self) {
        if self.pipeline_filters.is_empty() {
            return;
        }
        self.pipeline_selected = (self.pipeline_selected + 1) % self.pipeline_filters.len();
    }

    /// Toggle the highlighted filter on or off (check / uncheck the box).
    pub fn pipeline_toggle_selected(&mut self) {
        if let Some(filter) = self.pipeline_filters.get_mut(self.pipeline_selected) {
            filter.enabled = !filter.enabled;
            self.status_message = if filter.enabled {
                format!("Enabled {} 🐾", filter.name)
            } else {
                format!("Disabled {} 🐱", filter.name)
            };
        }
    }

    /// Type a character. On the filter list it edits the highlighted filter's
    /// params (only meaningful when that filter is enabled); on the file rows it
    /// edits the path.
    pub fn pipeline_input_char(&mut self, c: char) {
        match self.pipeline_field {
            PipelineField::Filters => {
                if let Some(filter) = self.pipeline_filters.get_mut(self.pipeline_selected) {
                    if filter.enabled {
                        filter.params.push(c);
                    }
                }
            }
            PipelineField::Input => self.pipeline_input_file.push(c),
            PipelineField::Output => self.pipeline_output_file.push(c),
        }
    }

    pub fn pipeline_input_backspace(&mut self) {
        match self.pipeline_field {
            PipelineField::Filters => {
                if let Some(filter) = self.pipeline_filters.get_mut(self.pipeline_selected) {
                    if filter.enabled {
                        filter.params.pop();
                    }
                }
            }
            PipelineField::Input => {
                self.pipeline_input_file.pop();
            }
            PipelineField::Output => {
                self.pipeline_output_file.pop();
            }
        }
    }

    /// Uncheck every filter and clear their params.
    pub fn pipeline_clear(&mut self) {
        for filter in &mut self.pipeline_filters {
            filter.enabled = false;
            filter.params.clear();
        }
        self.status_message = "Pipeline cleared 🐾".to_string();
    }

    /// Build the ordered list of enabled filters as pipeline steps.
    pub fn enabled_steps(&self) -> Vec<PipelineStep> {
        self.pipeline_filters
            .iter()
            .filter(|f| f.enabled)
            .map(|f| PipelineStep {
                filter: f.name.clone(),
                params: f.params.trim().to_string(),
            })
            .collect()
    }

    /// Validate and run the whole pipeline, applying each enabled filter in
    /// list order.
    pub fn pipeline_run_action(&mut self) {
        let steps = self.enabled_steps();

        if steps.is_empty() {
            self.status_message = "Check at least one filter before running 🐱".to_string();
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

        let result = run_pipeline(&input_path, &output_path, &steps);

        self.processing = false;
        self.status_message = match result {
            Ok(msg) => msg,
            Err(e) => format!("Error: {} 🐱", e),
        };
        // Leave the Processing screen now so the result message above is
        // actually shown, instead of run_app's is_processing_done() check
        // immediately firing go_to_main() and overwriting it.
        self.current_screen = Screen::Pipeline;
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
        // Crop then resize is not commutative: cropping a 20x20 region out of
        // the 40x40 source then resizing it down only works in this order. If
        // the steps ran in reverse, the image would already be 5x5 by the
        // time crop's 10,10,20,20 region is applied, which is out of bounds
        // and fails.
        let steps = vec![step("crop", "10,10,20,20"), step("resize", "5,5")];

        let msg = run_pipeline(
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            &steps,
        )
        .expect("pipeline should succeed");

        assert!(msg.contains("2 step(s)"));

        let saved = image::open(&output).unwrap();
        assert_eq!(saved.width(), 5);
        assert_eq!(saved.height(), 5);

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
        let output = out_path("validate");
        // The input path doesn't exist. If validation truly runs before any
        // decode attempt, we get the Step 2 validation error, not a "not
        // found" error from trying to open the (missing) file.
        let steps = vec![step("grayscale", ""), step("brightness", "notanumber")];

        let err = run_pipeline("/no/such/validate_input.png", output.to_str().unwrap(), &steps)
            .unwrap_err();

        assert!(err.contains("Step 2"), "error was: {}", err);
        assert!(!err.contains("not found"), "error was: {}", err);
        assert!(!output.exists());
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

    /// Index of a filter row in a fresh pipeline list, by name.
    fn filter_idx(app: &App, name: &str) -> usize {
        app.pipeline_filters
            .iter()
            .position(|f| f.name == name)
            .unwrap()
    }

    #[test]
    fn pipeline_list_mirrors_known_filters() {
        let app = App::new();
        assert_eq!(app.pipeline_filters.len(), crate::filters::KNOWN_FILTERS.len());
        // All start unchecked with empty params.
        assert!(app.pipeline_filters.iter().all(|f| !f.enabled));
        assert!(app.pipeline_filters.iter().all(|f| f.params.is_empty()));
    }

    #[test]
    fn toggle_checks_and_unchecks_selected() {
        let mut app = App::new();
        app.go_to_pipeline();
        assert!(!app.pipeline_filters[0].enabled);

        app.pipeline_toggle_selected();
        assert!(app.pipeline_filters[0].enabled);
        assert!(app.status_message.contains("Enabled"));

        app.pipeline_toggle_selected();
        assert!(!app.pipeline_filters[0].enabled);
        assert!(app.status_message.contains("Disabled"));
    }

    #[test]
    fn enabled_steps_keep_list_order_and_params() {
        let mut app = App::new();
        // Enable grayscale and brightness (with params), leave others off.
        let g = filter_idx(&app, "grayscale");
        let b = filter_idx(&app, "brightness");
        app.pipeline_filters[g].enabled = true;
        app.pipeline_filters[b].enabled = true;
        app.pipeline_filters[b].params = "  30  ".to_string();

        let steps = app.enabled_steps();
        assert_eq!(steps.len(), 2);
        // brightness comes before grayscale in KNOWN_FILTERS order.
        assert_eq!(steps[0].filter, "brightness");
        assert_eq!(steps[0].params, "30"); // trimmed
        assert_eq!(steps[1].filter, "grayscale");
    }

    #[test]
    fn params_only_edit_when_selected_filter_enabled() {
        let mut app = App::new();
        app.go_to_pipeline();
        app.pipeline_field = PipelineField::Filters;

        // Highlighted filter is disabled: typing is ignored.
        app.pipeline_input_char('9');
        assert!(app.pipeline_filters[app.pipeline_selected].params.is_empty());

        // Enable it, then typing edits its params.
        app.pipeline_toggle_selected();
        app.pipeline_input_char('4');
        app.pipeline_input_char('2');
        assert_eq!(app.pipeline_filters[app.pipeline_selected].params, "42");
        app.pipeline_input_backspace();
        assert_eq!(app.pipeline_filters[app.pipeline_selected].params, "4");
    }

    #[test]
    fn selection_navigation_wraps() {
        let mut app = App::new();
        assert_eq!(app.pipeline_selected, 0);
        app.pipeline_select_previous();
        assert_eq!(app.pipeline_selected, app.pipeline_filters.len() - 1);
        app.pipeline_select_next();
        assert_eq!(app.pipeline_selected, 0);
    }

    #[test]
    fn clear_unchecks_all_and_resets_params() {
        let mut app = App::new();
        app.pipeline_filters[0].enabled = true;
        app.pipeline_filters[0].params = "1.5".to_string();

        app.pipeline_clear();
        assert!(app.pipeline_filters.iter().all(|f| !f.enabled));
        assert!(app.pipeline_filters.iter().all(|f| f.params.is_empty()));
    }

    #[test]
    fn field_navigation_cycles() {
        let mut app = App::new();
        assert_eq!(app.pipeline_field, PipelineField::Filters);
        app.pipeline_next_field();
        assert_eq!(app.pipeline_field, PipelineField::Input);
        app.pipeline_next_field();
        assert_eq!(app.pipeline_field, PipelineField::Output);
        app.pipeline_prev_field();
        assert_eq!(app.pipeline_field, PipelineField::Input);
        app.pipeline_prev_field();
        assert_eq!(app.pipeline_field, PipelineField::Filters);
        app.pipeline_prev_field();
        assert_eq!(app.pipeline_field, PipelineField::Output);
    }

    #[test]
    fn run_action_requires_steps_and_files() {
        let mut app = App::new();
        // Nothing checked yet.
        app.pipeline_run_action();
        assert!(app.status_message.contains("at least one filter"));

        // A filter checked but no files.
        app.pipeline_filters[0].enabled = true;
        app.pipeline_run_action();
        assert!(app.status_message.contains("input and output"));
    }
}
