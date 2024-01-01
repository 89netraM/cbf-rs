use self::pixel::Pixels;

pub mod pixel;

pub struct Image<P> {
	pub width: usize,
	pub height: usize,
	pub pixels: Box<[P]>,
}

pub enum ImageEnum {
	U8(Image<u8>),
	I8(Image<i8>),
	U16(Image<u16>),
	I16(Image<i16>),
	U32(Image<u32>),
	I32(Image<i32>),
	F32(Image<f32>),
	U64(Image<u64>),
	I64(Image<i64>),
	F64(Image<f64>),
}

impl ImageEnum {
	pub fn from_pixels(width: usize, height: usize, pixels: Pixels) -> Self {
		macro_rules! from_pixels {
			($($name:ident),*) => {
				match pixels {
					$(Pixels::$name(pixels) => ImageEnum::$name(Image {
						width,
						height,
						pixels,
					}),)*
				}
			};
		}
		from_pixels!(U8, I8, U16, I16, U32, I32, F32, U64, I64, F64)
	}
}
