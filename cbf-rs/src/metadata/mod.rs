pub mod headers;

use std::{collections::HashMap, io::BufRead, str::FromStr};

use thiserror::Error as ThisError;

use headers::Error as HeadersError;

#[derive(Debug, ThisError)]
pub enum Error {
	#[error("invalid header value")]
	Parsing(ErrorKind),
	#[error(transparent)]
	Reading(#[from] HeadersError),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ErrorKind {
	InvalidContentType,
	InvalidConversion,
	InvalidContentTransferEncoding,
	InvalidEncoding,
	InvalidCharset,
	InvalidSize,
	InvalidPadding,
	InvalidByteOrder,
	InvalidMd5Digest,
	InvalidElementType,
	InvalidElementCount,
	InvalidWidth,
	InvalidHeight,
	InvalidDepth,
	MissingContentType,
	MissingContentTransferEncoding,
	MissingSize,
	MissingByteOrder,
	MissingElementType,
	MissingElementCount,
}

pub fn read_metadata(reader: impl BufRead) -> Result<Metadata, Error> {
	let headers = headers::read_headers(reader)?;
	parse_metadata(headers)
}

fn parse_metadata(headers: HashMap<String, String>) -> Result<Metadata, Error> {
	macro_rules! field {
		($field_name:literal) => {
			headers.get($field_name).map(|s| s.parse()).transpose()
		};
		($field_name:literal, $error:ident) => {
			field!($field_name).map_err(|_| Error::Parsing(ErrorKind::$error))
		};
	}
	macro_rules! required_field {
		($field_name:literal, $missing:ident) => {
			field!($field_name).and_then(|o| o.ok_or_else(|| Error::Parsing(ErrorKind::$missing)))
		};
		($field_name:literal, $error:ident, $missing:ident) => {
			field!($field_name, $error).and_then(|o| o.ok_or_else(|| Error::Parsing(ErrorKind::$missing)))
		};
	}

	Ok(Metadata {
		content_type: required_field!("content-type", MissingContentType)?,
		content_transfer_encoding: required_field!("content-transfer-encoding", MissingContentTransferEncoding)?,
		size: required_field!("x-binary-size", InvalidSize, MissingSize)?,
		padding: field!("x-binary-size-padding", InvalidPadding)?,
		byte_order: required_field!("x-binary-element-byte-order", MissingByteOrder)?,
		md5_digest: field!("content-md5", InvalidMd5Digest)?,
		element_type: required_field!("x-binary-element-type", MissingElementType)?,
		element_count: required_field!("x-binary-number-of-elements", InvalidElementCount, MissingElementCount)?,
		width: field!("x-binary-size-fastest-dimension", InvalidWidth)?,
		height: field!("x-binary-size-second-dimension", InvalidHeight)?,
		depth: field!("x-binary-size-third-dimension", InvalidDepth)?,
	})
}

#[derive(Debug)]
pub struct Metadata {
	pub content_type: ContentType,
	pub content_transfer_encoding: ContentTransferEncoding,
	pub size: usize,
	pub padding: Option<usize>,
	pub byte_order: ByteOrder,
	pub md5_digest: Option<String>,
	pub element_type: ElementType,
	pub element_count: usize,
	pub width: Option<usize>,
	pub height: Option<usize>,
	pub depth: Option<usize>,
}

#[derive(Debug)]
pub struct ContentType {
	pub mime_type: String,
	pub subtype: String,
	pub conversion: Option<Conversion>,
}

impl FromStr for ContentType {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let (type_, params) = s
			.split_once(';')
			.map(|(t, p)| (t, Some(p)))
			.unwrap_or_else(|| (s, None));

		let (mime_type, subtype) = type_
			.split_once('/')
			.ok_or_else(|| Error::Parsing(ErrorKind::InvalidContentType))?;

		let conversion = params
			.and_then(|p| parse_params_to_conversion(p).transpose())
			.transpose()?;

		Ok(ContentType { mime_type: mime_type.to_lowercase(), subtype: subtype.to_lowercase(), conversion })
	}
}

#[derive(Debug, PartialEq, Eq)]
pub enum Conversion {
	Packed(Option<PackedKind>),
	Canonical,
	ByteOffset,
	BackgroundOffsetDelta,
}

fn parse_params_to_conversion(params: &str) -> Result<Option<Conversion>, Error> {
	let mut conversion = None;
	let mut packed_kind = None;
	for param in params.to_lowercase().split(';') {
		let param = param.trim();
		if let Some(param) = param.strip_prefix("conversions=") {
			let param = param.trim_matches(is_whitespace_or_quote);
			conversion = match param {
				"x-cbf_packed" => Some(Conversion::Packed(None)),
				"x-cbf_canonical" => Some(Conversion::Canonical),
				"x-cbf_byte_offset" => Some(Conversion::ByteOffset),
				"x-cbf_background_offset_delta" => Some(Conversion::BackgroundOffsetDelta),
				_ => return Err(Error::Parsing(ErrorKind::InvalidConversion)),
			};
		} else if param.starts_with("uncorrelated_sections") {
			packed_kind = Some(PackedKind::UncorrelatedSections);
		} else if param.starts_with("flat") {
			packed_kind = Some(PackedKind::Flat);
		}
	}
	if matches!(conversion, Some(Conversion::Packed(_))) {
		conversion = Some(Conversion::Packed(packed_kind));
	}
	Ok(conversion)
}

#[derive(Debug, PartialEq, Eq)]
pub enum PackedKind {
	UncorrelatedSections,
	Flat,
}

#[derive(Debug)]
pub struct ContentTransferEncoding {
	pub encoding: Encoding,
	pub charset: Option<Charset>,
}

impl FromStr for ContentTransferEncoding {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let s = s.to_lowercase();
		let mut parts = s.split(';');
		let encoding = parts
			.next()
			.ok_or_else(|| Error::Parsing(ErrorKind::InvalidContentTransferEncoding))?
			.trim()
			.parse()?;
		let charset = parts
			.find(|s| s.trim().starts_with("charset="))
			.and_then(|s| s.trim().split('=').nth(1))
			.map(|s| s.trim_matches(is_whitespace_or_quote).parse())
			.transpose()?;
		Ok(ContentTransferEncoding { encoding, charset })
	}
}

#[derive(Debug, PartialEq, Eq)]
pub enum Encoding {
	Base8,
	Base10,
	Base16,
	Base32K,
	Base64,
	Binary,
	QuotedPrintable,
}

impl FromStr for Encoding {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_ref() {
			"x-base8" => Ok(Encoding::Base8),
			"x-base10" => Ok(Encoding::Base10),
			"x-base16" => Ok(Encoding::Base16),
			"x-base32k" => Ok(Encoding::Base32K),
			"base64" => Ok(Encoding::Base64),
			"binary" => Ok(Encoding::Binary),
			"quoted-printable" => Ok(Encoding::QuotedPrintable),
			_ => Err(Error::Parsing(ErrorKind::InvalidEncoding)),
		}
	}
}

#[derive(Debug, PartialEq, Eq)]
pub enum Charset {
	UsAscii,
	UTF8,
	UTF16,
}

impl FromStr for Charset {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_ref() {
			"us-ascii" => Ok(Charset::UsAscii),
			"utf-8" => Ok(Charset::UTF8),
			"utf-16" => Ok(Charset::UTF16),
			_ => Err(Error::Parsing(ErrorKind::InvalidCharset)),
		}
	}
}

#[derive(Debug, PartialEq, Eq)]
pub enum ByteOrder {
	LittleEndian,
	BigEndian,
}

impl FromStr for ByteOrder {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_ref() {
			"little_endian" => Ok(ByteOrder::LittleEndian),
			"big_endian" => Ok(ByteOrder::BigEndian),
			_ => Err(Error::Parsing(ErrorKind::InvalidByteOrder)),
		}
	}
}

#[derive(Debug, PartialEq, Eq)]
pub enum ElementType {
	Unsigned1bitInteger,
	Unsigned8bitInteger,
	Signed8bitInteger,
	Unsigned16bitInteger,
	Signed16bitInteger,
	Unsigned32bitInteger,
	Signed32bitInteger,
	Signed32bitReal,
	Signed64bitReal,
	Signed32bitComplex,
}

impl FromStr for ElementType {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_ref() {
			"unsigned 1-bit integer" => Ok(ElementType::Unsigned1bitInteger),
			"unsigned 8-bit integer" => Ok(ElementType::Unsigned8bitInteger),
			"signed 8-bit integer" => Ok(ElementType::Signed8bitInteger),
			"unsigned 16-bit integer" => Ok(ElementType::Unsigned16bitInteger),
			"signed 16-bit integer" => Ok(ElementType::Signed16bitInteger),
			"unsigned 32-bit integer" => Ok(ElementType::Unsigned32bitInteger),
			"signed 32-bit integer" => Ok(ElementType::Signed32bitInteger),
			"signed 32-bit real ieee" => Ok(ElementType::Signed32bitReal),
			"signed 64-bit real ieee" => Ok(ElementType::Signed64bitReal),
			"signed 32-bit complex ieee" => Ok(ElementType::Signed32bitComplex),
			_ => Err(Error::Parsing(ErrorKind::InvalidElementType)),
		}
	}
}

fn is_whitespace_or_quote(c: char) -> bool {
	c.is_whitespace() || c == '"'
}

#[cfg(test)]
mod tests {
	use std::io::Cursor;

	use super::{
		read_metadata, ByteOrder, Charset, ContentTransferEncoding, ContentType, Conversion, ElementType, Encoding,
		PackedKind,
	};

	#[test]
	fn test_real_metadata() {
		let header_text = "\
Content-Transfer-Encoding: BINARY\r
X-Binary-ID: 1\r
X-Binary-Element-Type: \"signed 32-bit integer\"\r
X-Binary-Element-Byte-Order: LITTLE_ENDIAN\r
X-Binary-Number-of-Elements: 8294400\r
X-Binary-Size-Fastest-Dimension: 2880\r
X-Binary-Size-Second-Dimension: 2880\r
X-Binary-Size-Padding: 1\r
Content-Type: application/octet-stream;\r
     conversions=\"x-CBF_BYTE_OFFSET\"\r
X-Binary-Size:   10161580\r
Content-MD5:     kL8G8UnwN1oKBdHWVkb0CQ==\r
\r\n";

		let metadata = read_metadata(Cursor::new(header_text)).expect("to parse real metadata");
		assert_eq!(
			metadata.content_type.mime_type,
			"application".to_owned(),
			"Content-Type mime type"
		);
		assert_eq!(
			metadata.content_type.subtype,
			"octet-stream".to_owned(),
			"Content-Type subtype"
		);
		assert_eq!(
			metadata.content_type.conversion,
			Some(Conversion::ByteOffset),
			"Content-Type conversion"
		);
		assert_eq!(
			metadata.content_transfer_encoding.encoding,
			Encoding::Binary,
			"Content-Transfer-Encoding"
		);
		assert_eq!(
			metadata.content_transfer_encoding.charset, None,
			"Content-Transfer-Encoding charset"
		);
		assert_eq!(metadata.size, 10161580, "Size");
		assert_eq!(metadata.padding, Some(1), "Padding");
		assert_eq!(metadata.byte_order, ByteOrder::LittleEndian, "Byte order");
		assert_eq!(
			metadata.md5_digest,
			Some("kL8G8UnwN1oKBdHWVkb0CQ==".to_owned()),
			"MD5 digest"
		);
		assert_eq!(metadata.element_type, ElementType::Signed32bitInteger, "Element type");
		assert_eq!(metadata.element_count, 8294400, "Element count");
		assert_eq!(metadata.width, Some(2880), "Width");
		assert_eq!(metadata.height, Some(2880), "Height");
		assert_eq!(metadata.depth, None, "Depth");
	}

	#[test]
	fn parse_content_type() {
		let content_type = "application/octet-stream"
			.parse::<ContentType>()
			.expect("to parse simple content type");
		assert_eq!(
			content_type.mime_type,
			"application".to_owned(),
			"Content-Type mime type"
		);
		assert_eq!(content_type.subtype, "octet-stream".to_owned(), "Content-Type subtype");
		assert_eq!(content_type.conversion, None, "Content-Type conversion");

		let content_type = "application/octet-stream;conversions=\"x-CBF_BYTE_OFFSET\""
			.parse::<ContentType>()
			.expect("to parse content type with conversion");
		assert_eq!(
			content_type.mime_type,
			"application".to_owned(),
			"Content-Type mime type"
		);
		assert_eq!(content_type.subtype, "octet-stream".to_owned(), "Content-Type subtype");
		assert_eq!(
			content_type.conversion,
			Some(Conversion::ByteOffset),
			"Content-Type conversion"
		);

		let content_type = "application/octet-stream;conversions=\"X-CBF_PACKED\";uncorrelated_sections"
			.parse::<ContentType>()
			.expect("to parse content type with conversion and packed parameter");
		assert_eq!(
			content_type.mime_type,
			"application".to_owned(),
			"Content-Type mime type"
		);
		assert_eq!(content_type.subtype, "octet-stream".to_owned(), "Content-Type subtype");
		assert_eq!(
			content_type.conversion,
			Some(Conversion::Packed(Some(PackedKind::UncorrelatedSections))),
			"Content-Type conversion"
		);

		let content_type = "application/octet-stream;flat;conversions=\"X-CBF_PACKED\""
			.parse::<ContentType>()
			.expect("to parse content type with conversion and packed parameter out of order");
		assert_eq!(
			content_type.mime_type,
			"application".to_owned(),
			"Content-Type mime type"
		);
		assert_eq!(content_type.subtype, "octet-stream".to_owned(), "Content-Type subtype");
		assert_eq!(
			content_type.conversion,
			Some(Conversion::Packed(Some(PackedKind::Flat))),
			"Content-Type conversion"
		);
	}

	#[test]
	fn parse_content_transfer_encoding() {
		let content_transfer_encoding = "BINARY"
			.parse::<ContentTransferEncoding>()
			.expect("to parse simple content transfer encoding");
		assert_eq!(
			content_transfer_encoding.encoding,
			Encoding::Binary,
			"Content-Transfer-Encoding"
		);
		assert_eq!(
			content_transfer_encoding.charset, None,
			"Content-Transfer-Encoding charset"
		);

		let content_transfer_encoding = "BINARY; charset=\"UTF-8\""
			.parse::<ContentTransferEncoding>()
			.expect("to parse content transfer encoding with charset");
		assert_eq!(
			content_transfer_encoding.encoding,
			Encoding::Binary,
			"Content-Transfer-Encoding"
		);
		assert_eq!(
			content_transfer_encoding.charset,
			Some(Charset::UTF8),
			"Content-Transfer-Encoding charset"
		);
	}

	#[test]
	fn parse_byte_order() {
		let byte_order = "LITTLE_ENDIAN"
			.parse::<ByteOrder>()
			.expect("to parse little endian byte order");
		assert_eq!(byte_order, ByteOrder::LittleEndian, "Byte order");

		let byte_order = "BIG_ENDIAN"
			.parse::<ByteOrder>()
			.expect("to parse big endian byte order");
		assert_eq!(byte_order, ByteOrder::BigEndian, "Byte order");
	}

	#[test]
	fn parse_element_type() {
		let element_type = "signed 32-bit integer"
			.parse::<ElementType>()
			.expect("to parse signed 32-bit integer element type");
		assert_eq!(element_type, ElementType::Signed32bitInteger, "Element type");

		let element_type = "unsigned 32-bit integer"
			.parse::<ElementType>()
			.expect("to parse unsigned 32-bit integer element type");
		assert_eq!(element_type, ElementType::Unsigned32bitInteger, "Element type");

		let element_type = "signed 32-bit real IEEE"
			.parse::<ElementType>()
			.expect("to parse signed 32-bit real IEEE element type");
		assert_eq!(element_type, ElementType::Signed32bitReal, "Element type");
	}
}
