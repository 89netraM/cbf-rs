pub struct Image {
	pub width: usize,
	pub height: usize,
	pub pixels: Pixels,
}

pub enum Pixels {
	U8(Box<[u8]>),
	I8(Box<[i8]>),
	U16(Box<[u16]>),
	I16(Box<[i16]>),
	U32(Box<[u32]>),
	I32(Box<[i32]>),
	F32(Box<[f32]>),
	U64(Box<[u64]>),
	I64(Box<[i64]>),
	F64(Box<[f64]>),
}

impl Image {
	pub fn new<P>(width: usize, height: usize, pixels: P) -> Self
	where
		P: Into<Pixels>,
	{
		Self { width, height, pixels: pixels.into() }
	}
}

macro_rules! impl_from_for_pixels {
	($($name:ident: $type:ty,)*) => {
		$(impl From<Vec<$type>> for Pixels {
			fn from(pixels: Vec<$type>) -> Self {
				Self::$name(pixels.into_boxed_slice())
			}
		})*
	};
}

impl_from_for_pixels!(
	U8: u8,
	I8: i8,
	U16: u16,
	I16: i16,
	U32: u32,
	I32: i32,
	F32: f32,
	U64: u64,
	I64: i64,
	F64: f64,
);
