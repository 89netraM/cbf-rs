use self::pixel::Pixels;

pub mod pixel;

pub struct Image<P> {
	pub width: usize,
	pub height: usize,
	pixels: Box<[P]>,
}

impl<P> Image<P> {
	pub fn get_pixel(&self, coordinate: impl ImageCoordinate) -> Option<&P> {
		Some(&self.pixels[coordinate.index(self.width, self.height)?])
	}

	pub fn pixels(&self) -> &[P] {
		&self.pixels
	}
}

pub trait ImageCoordinate {
	fn index(&self, width: usize, height: usize) -> Option<usize>;
}

impl ImageCoordinate for usize {
	fn index(&self, width: usize, height: usize) -> Option<usize> {
		if width * height - 1 < *self {
			None
		} else {
			Some(*self)
		}
	}
}

impl ImageCoordinate for (isize, isize) {
	fn index(&self, width: usize, height: usize) -> Option<usize> {
		let (x, y) = self;

		let x = x + (width / 2) as isize;
		let y = y + (height / 2) as isize;

		if x < 0 || y < 0 || width < x as usize || height < y as usize {
			return None;
		}

		let index = y as usize * width + x as usize;
		index.index(width, height)
	}
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
	pub fn width(&self) -> usize {
		match self {
			ImageEnum::U8(image) => image.width,
			ImageEnum::I8(image) => image.width,
			ImageEnum::U16(image) => image.width,
			ImageEnum::I16(image) => image.width,
			ImageEnum::U32(image) => image.width,
			ImageEnum::I32(image) => image.width,
			ImageEnum::F32(image) => image.width,
			ImageEnum::U64(image) => image.width,
			ImageEnum::I64(image) => image.width,
			ImageEnum::F64(image) => image.width,
		}
	}

	pub fn height(&self) -> usize {
		match self {
			ImageEnum::U8(image) => image.height,
			ImageEnum::I8(image) => image.height,
			ImageEnum::U16(image) => image.height,
			ImageEnum::I16(image) => image.height,
			ImageEnum::U32(image) => image.height,
			ImageEnum::I32(image) => image.height,
			ImageEnum::F32(image) => image.height,
			ImageEnum::U64(image) => image.height,
			ImageEnum::I64(image) => image.height,
			ImageEnum::F64(image) => image.height,
		}
	}

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
