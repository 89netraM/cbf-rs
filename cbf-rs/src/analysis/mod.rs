mod average;
pub mod sampler_methods;

use std::f64;

use crate::image::Image;

use self::average::{Average, BigNum};

pub fn radial_difraction_analysis<P: BigNum>(
	image: &Image<P>,
	config: &AnalysisConfig,
	mut sampler_method: impl FnMut(&Image<P>, f64, f64) -> Option<P>,
) -> Box<[P]> {
	let mut samples = allocate_slice(config.theta_sample_count);

	let rot = f64::consts::PI / (config.intensity_sample_count as f64);
	let rad = config.radius / (config.theta_sample_count as f64);
	for i in 0..config.intensity_sample_count {
		let angle = (i as f64) * rot;
		for j in 0..config.theta_sample_count {
			let r = (j as f64) * rad;
			if let Some(value) = sampler_method(image, angle, r) {
				samples[j].add(value);
			}
		}
	}

	compute_average_slice(samples)
}

pub struct AnalysisConfig {
	/// Points along radius
	theta_sample_count: usize,
	/// Points across radius
	intensity_sample_count: usize,
	/// Size of sample area
	radius: f64,
}

impl AnalysisConfig {
	pub fn new(theta_sample_count: usize, intensity_sample_count: usize, radius: f64) -> Option<Self> {
		if radius < 0.0 || f64::consts::SQRT_2 < radius {
			return None;
		}
		Some(Self { theta_sample_count, intensity_sample_count, radius })
	}
}

fn allocate_slice<P: BigNum>(len: usize) -> Box<[Average<P>]> {
	(0..len).map(|_| Average::default()).collect()
}

fn compute_average_slice<P: BigNum>(averages: Box<[Average<P>]>) -> Box<[P]> {
	averages.into_iter().map(Average::average).collect()
}

#[cfg(test)]
mod tests {
	use super::{radial_difraction_analysis, sampler_methods::nearest_neighbour, AnalysisConfig};
	use crate::{image::ImageEnum, read_image};

	use std::f64;
	use std::io::Cursor;

	#[test]
	fn analyse_real_image() {
		const EXAMPLE_DATA: &'static [u8] = include_bytes!("../examples/snap_V4_00013.cbf");
		let mut reader = Cursor::new(EXAMPLE_DATA);
		let image = read_image(&mut reader).expect("to read real image");
		let ImageEnum::I64(image) = image else {
			panic!("expected i64 pixels")
		};

		let Some(config) = AnalysisConfig::new(2160, 1000, f64::consts::SQRT_2) else {
			panic!("expected analysis config to be valid")
		};

		let analysis = radial_difraction_analysis(&image, &config, nearest_neighbour);
		println!("{:?}", analysis);
	}
}
