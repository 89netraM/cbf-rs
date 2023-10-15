/// Trait for converting little endian byte arrays to a primitives.
pub trait FromBytes: Sized {
	fn from_1_bytes(bytes: [u8; 1]) -> Self;
	fn from_2_bytes(bytes: [u8; 2]) -> Self;
	fn from_4_bytes(bytes: [u8; 4]) -> Self;
	fn from_8_bytes(bytes: [u8; 8]) -> Self;
}

impl FromBytes for u8 {
	fn from_1_bytes([byte]: [u8; 1]) -> Self {
		byte
	}

	fn from_2_bytes([byte, ..]: [u8; 2]) -> Self {
		byte
	}

	fn from_4_bytes([byte, ..]: [u8; 4]) -> Self {
		byte
	}

	fn from_8_bytes([byte, ..]: [u8; 8]) -> Self {
		byte
	}
}

impl FromBytes for i8 {
	fn from_1_bytes([byte]: [u8; 1]) -> Self {
		byte as i8
	}

	fn from_2_bytes([byte, ..]: [u8; 2]) -> Self {
		byte as i8
	}

	fn from_4_bytes([byte, ..]: [u8; 4]) -> Self {
		byte as i8
	}

	fn from_8_bytes([byte, ..]: [u8; 8]) -> Self {
		byte as i8
	}
}

impl FromBytes for u16 {
	fn from_1_bytes([byte]: [u8; 1]) -> Self {
		byte as u16
	}

	fn from_2_bytes(bytes: [u8; 2]) -> Self {
		u16::from_le_bytes(bytes)
	}

	fn from_4_bytes([byte1, byte2, ..]: [u8; 4]) -> Self {
		u16::from_le_bytes([byte1, byte2])
	}

	fn from_8_bytes([byte1, byte2, ..]: [u8; 8]) -> Self {
		u16::from_le_bytes([byte1, byte2])
	}
}

impl FromBytes for i16 {
	fn from_1_bytes([byte]: [u8; 1]) -> Self {
		byte as i8 as i16
	}

	fn from_2_bytes(bytes: [u8; 2]) -> Self {
		i16::from_le_bytes(bytes)
	}

	fn from_4_bytes([byte1, byte2, ..]: [u8; 4]) -> Self {
		i16::from_le_bytes([byte1, byte2])
	}

	fn from_8_bytes([byte1, byte2, ..]: [u8; 8]) -> Self {
		i16::from_le_bytes([byte1, byte2])
	}
}

impl FromBytes for u32 {
	fn from_1_bytes([byte]: [u8; 1]) -> Self {
		byte as u32
	}

	fn from_2_bytes(bytes: [u8; 2]) -> Self {
		u16::from_le_bytes(bytes) as u32
	}

	fn from_4_bytes(bytes: [u8; 4]) -> Self {
		u32::from_le_bytes(bytes)
	}

	fn from_8_bytes([byte1, byte2, byte3, byte4, ..]: [u8; 8]) -> Self {
		u32::from_le_bytes([byte1, byte2, byte3, byte4])
	}
}

impl FromBytes for i32 {
	fn from_1_bytes([byte]: [u8; 1]) -> Self {
		byte as i8 as i32
	}

	fn from_2_bytes(bytes: [u8; 2]) -> Self {
		i16::from_le_bytes(bytes) as i32
	}

	fn from_4_bytes(bytes: [u8; 4]) -> Self {
		i32::from_le_bytes(bytes)
	}

	fn from_8_bytes([byte1, byte2, byte3, byte4, ..]: [u8; 8]) -> Self {
		i32::from_le_bytes([byte1, byte2, byte3, byte4])
	}
}

impl FromBytes for f32 {
	fn from_1_bytes([byte]: [u8; 1]) -> Self {
		byte as i8 as f32
	}

	fn from_2_bytes(bytes: [u8; 2]) -> Self {
		i16::from_le_bytes(bytes) as f32
	}

	fn from_4_bytes(bytes: [u8; 4]) -> Self {
		f32::from_le_bytes(bytes)
	}

	fn from_8_bytes([byte1, byte2, byte3, byte4, ..]: [u8; 8]) -> Self {
		f32::from_le_bytes([byte1, byte2, byte3, byte4])
	}
}

impl FromBytes for u64 {
	fn from_1_bytes([byte]: [u8; 1]) -> Self {
		byte as u64
	}

	fn from_2_bytes(bytes: [u8; 2]) -> Self {
		u16::from_le_bytes(bytes) as u64
	}

	fn from_4_bytes(bytes: [u8; 4]) -> Self {
		u32::from_le_bytes(bytes) as u64
	}

	fn from_8_bytes(bytes: [u8; 8]) -> Self {
		u64::from_le_bytes(bytes)
	}
}

impl FromBytes for i64 {
	fn from_1_bytes([byte]: [u8; 1]) -> Self {
		byte as i8 as i64
	}

	fn from_2_bytes(bytes: [u8; 2]) -> Self {
		i16::from_le_bytes(bytes) as i64
	}

	fn from_4_bytes(bytes: [u8; 4]) -> Self {
		i32::from_le_bytes(bytes) as i64
	}

	fn from_8_bytes(bytes: [u8; 8]) -> Self {
		i64::from_le_bytes(bytes)
	}
}

impl FromBytes for f64 {
	fn from_1_bytes([byte]: [u8; 1]) -> Self {
		byte as i8 as f64
	}

	fn from_2_bytes(bytes: [u8; 2]) -> Self {
		i16::from_le_bytes(bytes) as f64
	}

	fn from_4_bytes(bytes: [u8; 4]) -> Self {
		f32::from_le_bytes(bytes) as f64
	}

	fn from_8_bytes(bytes: [u8; 8]) -> Self {
		f64::from_le_bytes(bytes)
	}
}
