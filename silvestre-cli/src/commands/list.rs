use crate::Result;

pub struct ListCommand;

impl ListCommand {
    pub fn execute() -> Result<()> {
        println!("Available Filters:");
        println!();

        println!("Effects:");
        println!("  brightness    Adjust brightness (delta: -255 to 255)");
        println!("  contrast      Adjust contrast (factor: 0.0 to ∞, default 1.0)");
        println!("  grayscale     Convert to grayscale");
        println!("  invert        Invert colors");
        println!();

        println!("Filters:");
        println!("  box-blur      Box blur filter (kernel-size: 1+)");
        println!("  canny         Canny edge detection");
        println!("  gaussian      Gaussian blur filter (sigma: 0.1+)");
        println!("  median        Median filter (kernel-size: 1+)");
        println!("  sharpen       Sharpen filter");
        println!("  sobel         Sobel edge detection");
        println!();

        println!("Transforms:");
        println!("  crop          Crop image (x, y, width, height)");
        println!("  mirror        Flip/mirror image (mode: h|v|both)");
        println!("  resize        Resize image (width, height, interpolation: nearest|bilinear)");
        println!("  rotate        Rotate image (angle: 0-360 or arbitrary with bilinear)");
        println!();

        println!("Analysis:");
        println!("  histogram     Compute image histogram");
        println!();

        println!("Use 'silvestre apply <filter> --help' for filter-specific options.");

        Ok(())
    }
}
