use cbf_rs::{
	image::{Image as CbfImage, Pixels},
	read_image,
};
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub struct Image(CbfImage);

#[wasm_bindgen]
impl Image {
	pub fn load(file: &[u8]) -> Result<Image, String> {
		let cbf_image = read_image(file).map_err(|e| format!("{e:?}"))?;
		Ok(Image(cbf_image))
	}

	#[wasm_bindgen(getter)]
	pub fn height(&self) -> usize {
		self.0.height
	}

	#[wasm_bindgen(getter)]
	pub fn width(&self) -> usize {
		self.0.width
	}

	#[wasm_bindgen(js_name = "writeImage")]
	pub fn write_image(&self, pixel_buffer: &mut [u8]) {
		match &self.0.pixels {
			Pixels::U8(slice) => write_image::u8(slice, pixel_buffer),
			Pixels::I8(slice) => write_image::i8(slice, pixel_buffer),
			Pixels::U16(slice) => write_image::u16(slice, pixel_buffer),
			Pixels::I16(slice) => write_image::i16(slice, pixel_buffer),
			Pixels::U32(slice) => write_image::u32(slice, pixel_buffer),
			Pixels::I32(slice) => write_image::i32(slice, pixel_buffer),
			Pixels::F32(_) => unimplemented!(),
			Pixels::U64(slice) => write_image::u64(slice, pixel_buffer),
			Pixels::I64(slice) => write_image::i64(slice, pixel_buffer),
			Pixels::F64(_) => unimplemented!(),
		}
	}
}

mod write_image {
	macro_rules! impl_write_image_for_pixels {
		($($name:ident: $type:ty,)*) => {
			$(pub fn $name(slice: &Box<[$type]>, pixel_buffer: &mut [u8]) {
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
		u64: u64,
		i64: i64,
	}

	fn min_max<N: Copy + Ord>(iter: impl Iterator<Item = N>) -> Option<(N, N)> {
		iter.fold(None, |a, n| match a {
			Some((min, max)) => Some((min.min(n), max.max(n))),
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
}