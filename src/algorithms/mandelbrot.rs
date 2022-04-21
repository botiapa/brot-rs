#[cfg(feature = "opencl")]
use ocl::OclPrm;

pub type Float = f64;

const DEFAULT_MAX_ITER: Float = 180.0;

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(C)]
pub struct FractalProperties {
    pub center_x: Float,
    pub center_y: Float,
    pub zoom: Float,
    pub max_iter: Float,
    pub ss_factor: i32,
    pub color_offset: Float,
    pub color_saturation: Float,
}

#[cfg(feature = "opencl")]
unsafe impl OclPrm for FractalProperties {}

impl Default for FractalProperties {
    fn default() -> Self {
        Self {
            center_x: 0 as Float,
            center_y: 0 as Float,
            zoom: 0.5 as Float,
            max_iter: DEFAULT_MAX_ITER,
            ss_factor: 1,
            color_offset: 10.0,
            color_saturation: 0.6,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AlgorithmType {
    NaiveCPU,
    #[cfg(feature = "opencl")]
    OpenCL,
}

/// Convert a given dimension onto the complex plane the following way:
/// - Map the position -> `[0;1]`
/// - Offset the range by `-0.5` essentially centering it (the center becomes 0) -> `[-0.5;0.5]`
/// - Expand the range -> `[-1;1]`
/// - Scale the range with the given zoom -> `[-1 / zoom;1 / zoom]`
/// - Offset the range with the given center point (move the center) -> `[center + (-1 / zoom);center + (1 / zoom)]`
pub fn map_to_complex_plane(n: Float, max_n: Float, center: Float, zoom: Float) -> Float {
    center + (((n / max_n) - 0.5) * 2 as Float / zoom)
}
