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

macro_rules! pixels_from_vec {
	($($name:ident: $type:ty,)*) => {
		$(impl From<Vec<$type>> for Pixels {
			fn from(value: Vec<$type>) -> Self {
				Pixels::$name(value.into())
			}
		})*
	};
}

pixels_from_vec!(
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
