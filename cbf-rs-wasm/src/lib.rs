use cbf_rs::{image::ImageEnum, read_image};
use std::cmp::Ordering;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub struct Image(ImageEnum);

#[wasm_bindgen]
impl Image {
	pub fn load(file: &[u8]) -> Result<Image, String> {
		let cbf_image = read_image(file).map_err(|e| format!("{e:?}"))?;
		Ok(Image(cbf_image))
	}

	#[wasm_bindgen(getter)]
	pub fn height(&self) -> usize {
		self.0.height()
	}

	#[wasm_bindgen(getter)]
	pub fn width(&self) -> usize {
		self.0.width()
	}

	#[wasm_bindgen(js_name = "writeImage")]
	pub fn write_image(&self, pixel_buffer: &mut [u8]) {
		match &self.0 {
			ImageEnum::U8(image) => write_image::u8(image.pixels(), pixel_buffer),
			ImageEnum::I8(image) => write_image::i8(image.pixels(), pixel_buffer),
			ImageEnum::U16(image) => write_image::u16(image.pixels(), pixel_buffer),
			ImageEnum::I16(image) => write_image::i16(image.pixels(), pixel_buffer),
			ImageEnum::U32(image) => write_image::u32(image.pixels(), pixel_buffer),
			ImageEnum::I32(image) => write_image::i32(image.pixels(), pixel_buffer),
			ImageEnum::F32(image) => write_image::f32(image.pixels(), pixel_buffer),
			ImageEnum::U64(image) => write_image::u64(image.pixels(), pixel_buffer),
			ImageEnum::I64(image) => write_image::i64(image.pixels(), pixel_buffer),
			ImageEnum::F64(image) => write_image::f64(image.pixels(), pixel_buffer),
		}
	}
}

#[wasm_bindgen]
pub struct Analysis(Vec<f64>);

#[wasm_bindgen]
impl Analysis {
	pub fn init() -> Analysis {
		Analysis(Vec::new())
	}

	pub fn analyze(&mut self, image: &Image) {
		match &image.0 {
			ImageEnum::U8(image) => analyze_image::u8(image, &mut self.0),
			ImageEnum::I8(image) => analyze_image::i8(image, &mut self.0),
			ImageEnum::U16(image) => analyze_image::u16(image, &mut self.0),
			ImageEnum::I16(image) => analyze_image::i16(image, &mut self.0),
			ImageEnum::U32(image) => analyze_image::u32(image, &mut self.0),
			ImageEnum::I32(image) => analyze_image::i32(image, &mut self.0),
			ImageEnum::F32(image) => analyze_image::f32(image, &mut self.0),
			ImageEnum::U64(image) => analyze_image::u64(image, &mut self.0),
			ImageEnum::I64(image) => analyze_image::i64(image, &mut self.0),
			ImageEnum::F64(image) => analyze_image::f64(image, &mut self.0),
		};
	}

	#[wasm_bindgen(getter)]
	pub fn raw(&self) -> Box<[f64]> {
		self.0.clone().into()
	}

	#[wasm_bindgen(getter, js_name = "localScaled")]
	pub fn local_scaled(&self) -> Box<[u8]> {
		let (min, max) = min_max(self.0.iter()).unwrap_or_else(|| (&f64::MIN, &f64::MAX));
		let magnitude = max - min;
		let scale = 255.0 / magnitude as f64;
		self.0
			.iter()
			.map(|n| ((*n - min) * scale) as u8)
			.flat_map(|v| [255 - v, 255 - v, 255 - v, 255])
			.collect()
	}
}

mod write_image {
	use super::{min_max, write_to_pixel_buffer};

	macro_rules! impl_write_image_for_pixels {
		($($name:ident: $type:ty,)*) => {
			$(pub fn $name(slice: &[$type], pixel_buffer: &mut [u8]) {
				let (min, max) = min_max(slice.iter()).unwrap_or_else(|| (&<$type>::MIN, &<$type>::MAX));
				let magnitude = max - min;
				let scale = 255.0 / magnitude as f64;
				let pixels = slice.iter().map(|n| ((*n - min) as f64 * scale) as u8);
				write_to_pixel_buffer(pixels, pixel_buffer);
			})*
		};
	}

	impl_write_image_for_pixels! {
		u8: u8,
		i8: i8,
		u16: u16,
		i16: i16,
		u32: u32,
		i32: i32,
		f32: f32,
		u64: u64,
		i64: i64,
		f64: f64,
	}
}

mod analyze_image {
	use cbf_rs::{
		analysis::{radial_difraction_analysis, sampler_methods::nearest_neighbour, AnalysisConfig},
		image::Image,
	};
	use std::f64;

	macro_rules! impl_analyze_image_for_pixels {
		($($name:ident: $type:ty,)*) => {
			$(pub fn $name(image: &Image<$type>, target: &mut impl Extend<f64>) {
				let result = radial_difraction_analysis(&image, &config_for_image(&image), nearest_neighbour);
				target.extend(result.into_iter().map(|n| *n as f64))
			})*
		};
	}

	fn config_for_image<P>(image: &Image<P>) -> AnalysisConfig {
		AnalysisConfig::new(image.width / 2, 1000, f64::consts::SQRT_2).unwrap()
	}

	impl_analyze_image_for_pixels! {
		u8: u8,
		i8: i8,
		u16: u16,
		i16: i16,
		u32: u32,
		i32: i32,
		f32: f32,
		u64: u64,
		i64: i64,
		f64: f64,
	}
}

fn min_max<N: Copy + PartialOrd>(iter: impl Iterator<Item = N>) -> Option<(N, N)> {
	iter.fold(None, |a, n| match a {
		Some((min, max)) => {
			let min = if n.partial_cmp(&min) == Some(Ordering::Less) {
				n
			} else {
				min
			};
			let max = if n.partial_cmp(&max) == Some(Ordering::Greater) {
				n
			} else {
				max
			};
			Some((min, max))
		}
		None => Some((n, n)),
	})
}

fn write_to_pixel_buffer(pixels: impl Iterator<Item = u8>, pixel_buffer: &mut [u8]) {
	for (i, v) in pixels.take(pixel_buffer.len() / 4).enumerate() {
		pixel_buffer[i * 4 + 0] = 255 - v;
		pixel_buffer[i * 4 + 1] = 255 - v;
		pixel_buffer[i * 4 + 2] = 255 - v;
		pixel_buffer[i * 4 + 3] = 255;
	}
}
