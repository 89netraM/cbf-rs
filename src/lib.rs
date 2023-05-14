pub mod compression;
pub mod image;
pub mod metadata;

use std::io::{BufRead, Error as IOError, Read};

use compression::read_byte_offset;
use thiserror::Error as ThisError;

use image::{Image, Pixels};
use metadata::{read_metadata, ByteOrder, Conversion, ElementType, Encoding, Error as MetadataError, Metadata};

pub fn read_all_images(mut reader: impl BufRead) -> Result<Vec<Image>, Error> {
	let mut images = Vec::new();

	while let Some(image) = try_read_next_image(&mut reader)? {
		images.push(image);
	}

	Ok(images)
}

fn try_read_next_image(reader: impl BufRead) -> Result<Option<Image>, Error> {
	match read_image(reader) {
		Ok(image) => Ok(Some(image)),
		Err(Error::NoImage) => Ok(None),
		Err(error) => Err(error),
	}
}

pub fn read_image(mut reader: impl BufRead) -> Result<Image, Error> {
	progress_reader_to_cbf_start(&mut reader)?;
	let metadata = read_metadata(&mut reader)?;
	read_binary_header(&mut reader)?;
	let pixels = read_pixels(&mut reader, &metadata)?;
	progress_reader_to_cbf_end(&mut reader)?;
	Ok(Image {
		width: metadata.width.ok_or(Error::MissingDimension)?,
		height: metadata.height.ok_or(Error::MissingDimension)?,
		pixels,
	})
}

fn read_pixels(reader: impl Read, metadata: &Metadata) -> Result<Pixels, Error> {
	if metadata.byte_order != ByteOrder::LittleEndian {
		return Err(Error::UnsupportedByteOrder);
	}
	if metadata.content_type.mime_type != "application" || metadata.content_type.subtype != "octet-stream" {
		return Err(Error::UnsupportedContentType);
	}
	if metadata.content_transfer_encoding.encoding != Encoding::Binary {
		return Err(Error::UnsupportedEncoding);
	}
	match metadata.content_type.conversion {
		Some(Conversion::ByteOffset) => {
			macro_rules! read_byte_offset {
				($($name:ident: $zero:expr,)*) => {
					match metadata.element_type {
						$(ElementType::$name => {
							let mut pixels = vec![$zero; metadata.element_count];
							read_byte_offset(reader, &mut pixels)?;
							Ok(pixels.into())
						})*
						_ => Err(Error::UnsupportedPixelFormat),
					}
				};
			}
			read_byte_offset!(
				Unsigned8bitInteger: 0u8,
				Signed8bitInteger: 0i8,
				Unsigned16bitInteger: 0u16,
				Signed16bitInteger: 0i16,
				Unsigned32bitInteger: 0u32,
				Signed32bitInteger: 0i32,
			)
		}
		_ => Err(Error::UnsupportedCompression),
	}
}

fn progress_reader_to_cbf_start(reader: impl BufRead) -> Result<(), Error> {
	match progress_reader_to(reader, "--CIF-BINARY-FORMAT-SECTION--\r\n")? {
		Reached::Needle => Ok(()),
		Reached::End => Err(Error::NoImage),
	}
}

fn progress_reader_to_cbf_end(reader: impl BufRead) -> Result<(), Error> {
	progress_reader_to(reader, "--CIF-BINARY-FORMAT-SECTION----\r\n")?;
	Ok(())
}

fn progress_reader_to(mut reader: impl BufRead, needle: &str) -> Result<Reached, Error> {
	let mut line = String::new();

	loop {
		line.clear();

		let bytes_read = reader.read_line(&mut line)?;

		if bytes_read == 0 {
			return Ok(Reached::End);
		}

		if line == needle {
			return Ok(Reached::Needle);
		}
	}
}

fn read_binary_header(mut reader: impl Read) -> Result<(), Error> {
	let mut header = [0; 4];

	reader.read_exact(&mut header)?;

	if header != [0x0C, 0x1A, 0x04, 0xD5] {
		return Err(Error::UnrecognisedBinaryHeader);
	}

	Ok(())
}

enum Reached {
	Needle,
	End,
}

#[derive(Debug, ThisError)]
pub enum Error {
	#[error(transparent)]
	Metadata(#[from] MetadataError),
	#[error(transparent)]
	IO(#[from] IOError),
	#[error("no image found")]
	NoImage,
	#[error("unsupported compression")]
	UnsupportedCompression,
	#[error("unsupported byte order")]
	UnsupportedByteOrder,
	#[error("unsupported pixel format")]
	UnsupportedPixelFormat,
	#[error("unsupported content type")]
	UnsupportedContentType,
	#[error("unsupported encoding")]
	UnsupportedEncoding,
	#[error("unrecognised binary header")]
	UnrecognisedBinaryHeader,
	#[error("missing dimension")]
	MissingDimension,
}

#[cfg(test)]
mod tests {
	use std::io::{Cursor, Read};

	use super::{image::Pixels, read_image};

	#[test]
	fn read_real_image() {
		const EXAMPLE_DATA: &'static [u8] = include_bytes!("./examples/snap_V4_00013.cbf");
		let mut reader = Cursor::new(EXAMPLE_DATA);
		let image = read_image(&mut reader).expect("to read real image");

		assert_eq!(image.width, 2880);
		assert_eq!(image.height, 2880);
		let Pixels::I32(pixels) = image.pixels else { panic!("expected i32 pixels") };
		assert_eq!(pixels[0], 100);
		assert_eq!(pixels[1], 100);
		assert_eq!(pixels[2880], 192);
		assert_eq!(pixels[4145760], 366);
		assert_eq!(pixels[4153200], 9636);
		assert_eq!(pixels[8294399], 100);

		let mut rest = String::new();
		reader.read_to_string(&mut rest).expect("to read rest as string");
		assert_eq!(rest, ";\r\n");
	}
}
