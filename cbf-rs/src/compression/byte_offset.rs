use std::{
	io::{Read, Result},
	ops::AddAssign,
};

use super::from_bytes::FromBytes;

pub fn read_byte_offset<P>(reader: impl Read, buf: &mut [P]) -> Result<()>
where
	P: FromBytes + AddAssign + Copy,
{
	let mut byte_offset_reader = ByteOffsetReader::new(reader);
	byte_offset_reader.read(buf)
}

struct ByteOffsetReader<R, P> {
	reader: R,
	base_value: P,
}

impl<R, P> ByteOffsetReader<R, P>
where
	P: FromBytes,
{
	fn new(reader: R) -> Self {
		Self { reader, base_value: P::from_1_bytes([0]) }
	}
}

impl<R, P> ByteOffsetReader<R, P>
where
	R: Read,
	P: FromBytes + AddAssign + Copy,
{
	pub fn read(&mut self, buf: &mut [P]) -> Result<()> {
		for i in 0..buf.len() {
			match self.read_value() {
				Ok(value) => buf[i] = value,
				Err(e) => return Err(e),
			}
		}
		Ok(())
	}

	fn read_value(&mut self) -> Result<P> {
		let value = read_value(&mut self.reader)?;
		self.base_value += value;
		Ok(self.base_value)
	}
}

fn read_value<P: FromBytes>(mut reader: impl Read) -> Result<P> {
	let bytes = read_n_bytes::<1>(&mut reader)?;
	if u8::from_1_bytes(bytes) != 0x80 {
		return Ok(P::from_1_bytes(bytes));
	}
	let bytes = read_n_bytes::<2>(&mut reader)?;
	if u16::from_2_bytes(bytes) != 0x8000 {
		return Ok(P::from_2_bytes(bytes));
	}
	let bytes = read_n_bytes::<4>(&mut reader)?;
	if u32::from_4_bytes(bytes) != 0x80000000 {
		return Ok(P::from_4_bytes(bytes));
	}
	let bytes = read_n_bytes::<8>(&mut reader)?;
	Ok(P::from_8_bytes(bytes))
}

fn read_n_bytes<const N: usize>(mut reader: impl Read) -> Result<[u8; N]> {
	let mut data = [0; N];
	reader.read_exact(&mut data)?;
	Ok(data)
}

#[cfg(test)]
mod tests {
	use std::io::Cursor;

	use super::{read_byte_offset, ByteOffsetReader};

	#[test]
	fn test_real_binary() {
		const EXAMPLE_DATA: &'static [u8] = include_bytes!("./examples/byte_offset.bin");
		let mut reader = Cursor::new(EXAMPLE_DATA);
		let mut buf = vec![0i32; 8294400];
		read_byte_offset(&mut reader, &mut buf).expect("to successfully read");
		assert_eq!(buf[0], 100);
		assert_eq!(buf[1], 100);
		assert_eq!(buf[2880], 192);
		assert_eq!(buf[4145760], 366);
		assert_eq!(buf[4153200], 9636);
		assert_eq!(buf[8294399], 100);
		assert_eq!(reader.position(), 10161580);
	}

	#[test]
	fn read_reader_as_8_bits() {
		let mut reader = Cursor::new([0x42]);
		let mut byte_offset_reader = ByteOffsetReader::<_, i32>::new(&mut reader);
		assert_eq!(byte_offset_reader.read_value().expect("to successfully read"), 0x42);
	}

	#[test]
	fn read_reader_as_16_bits() {
		let mut reader = Cursor::new([0x80, 0x20, 0x04]);
		let mut byte_offset_reader = ByteOffsetReader::<_, i32>::new(&mut reader);
		assert_eq!(byte_offset_reader.read_value().expect("to successfully read"), 0x0420);
	}

	#[test]
	fn read_reader_as_32_bits() {
		let mut reader = Cursor::new([0x80, 0x00, 0x80, 0x20, 0x04, 0x20, 0x04]);
		let mut byte_offset_reader = ByteOffsetReader::<_, i32>::new(&mut reader);
		assert_eq!(
			byte_offset_reader.read_value().expect("to successfully read"),
			0x04200420
		);
	}

	#[test]
	fn read_reader_as_64_bits() {
		let mut reader = Cursor::new([
			0x80, 0x00, 0x80, 0x00, 0x00, 0x00, 0x80, 0x20, 0x04, 0x20, 0x04, 0x20, 0x04, 0x20, 0x04,
		]);
		let mut byte_offset_reader = ByteOffsetReader::<_, i64>::new(&mut reader);
		assert_eq!(
			byte_offset_reader.read_value().expect("to successfully read"),
			0x0420042004200420
		);
	}

	#[test]
	fn combine_with_base_value() {
		let mut reader = Cursor::new([0x42, 0x24]);
		let mut byte_offset_reader = ByteOffsetReader::<_, i32>::new(&mut reader);
		assert_eq!(byte_offset_reader.read_value().expect("to successfully read"), 0x42);
		assert_eq!(byte_offset_reader.read_value().expect("to successfully read"), 0x66);
	}
}
