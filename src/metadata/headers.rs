use std::{
	borrow::Cow,
	collections::HashMap,
	io::{BufRead, Error as IOError, ErrorKind as IOErrorKind},
};

use nom::{
	branch::alt,
	bytes::streaming::{escaped, take_until, take_while, take_while1},
	character::streaming::{anychar, char, crlf, space0},
	combinator::opt,
	error::Error as NomError,
	multi::fold_many1,
	sequence::{delimited, pair, preceded, separated_pair, terminated},
	Err, IResult,
};
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
	#[error("invalid header format")]
	Parsing(#[from] Err<NomError<String>>),
	#[error(transparent)]
	IO(#[from] IOError),
}

pub fn read_headers(mut reader: impl BufRead) -> Result<HashMap<String, String>, Error> {
	let mut headers = HashMap::new();

	let mut line = String::new();
	reader.read_line(&mut line)?;
	while !line.is_empty() && line != "\r\n" {
		let (key, value) = read_header(&mut reader, &mut line)?;
		headers.insert(key, value);
	}

	Ok(headers)
}

fn read_header(mut reader: impl BufRead, line: &mut String) -> Result<(String, String), Error> {
	match field(line) {
		Ok((rest, (key, value))) => {
			let result = (key.to_owned(), value.into_owned());
			*line = rest.to_owned();
			Ok(result)
		}
		Err(Err::Incomplete(_)) => {
			if reader.read_line(line)? == 0 {
				return Err(Error::IO(IOErrorKind::UnexpectedEof.into()));
			}
			read_header(reader, line)
		}
		Err(error) => Err(error.map_input(str::to_owned).into()),
	}
}

fn field(input: &str) -> IResult<&str, (&str, Cow<'_, str>)> {
	terminated(
		separated_pair(field_name, delimited(space0, char(':'), space0), field_body),
		crlf,
	)(input)
}

fn field_name(input: &str) -> IResult<&str, &str> {
	take_while1(|c| ' ' <= c && c != ' ' && c != ':' && c <= '~')(input)
}

fn field_body(input: &str) -> IResult<&str, Cow<'_, str>> {
	let (rest, (mut body, body_cont)) =
		pair(field_body_contents, opt(preceded(pair(crlf, lwsp_chars), field_body)))(input)?;
	if let Some(cont) = body_cont {
		body.to_mut().push_str(&cont);
	}
	Ok((rest, body))
}

fn field_body_contents(input: &str) -> IResult<&str, Cow<'_, str>> {
	let (rest, content) = alt((quoted_string, text))(input)?;
	Ok((rest, content.into()))
}

fn quoted_string(input: &str) -> IResult<&str, &str> {
	delimited(
		char('"'),
		escaped(take_while(|c| c != '"' && c != '\\' && c != '\r'), '\\', anychar),
		char('"'),
	)(input)
}

fn text(input: &str) -> IResult<&str, &str> {
	take_until("\r\n")(input)
}

fn lwsp_chars(input: &str) -> IResult<&str, ()> {
	fold_many1(alt((char(' '), char('\t'))), || (), |_, _| ())(input)
}

#[cfg(test)]
mod tests {
	use std::io::Cursor;

	use nom::{
		error::{Error, ErrorKind::*},
		Err::*,
		Needed::*,
	};

	use super::{field_body, field_body_contents, field_name, read_headers};

	#[test]
	fn test_real_headers() {
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
		let mut reader = Cursor::new(header_text);
		let headers = read_headers(&mut reader).expect("to parse real headers");
		assert_eq!(reader.position(), header_text.len() as u64);
		assert_eq!(headers.len(), 11);
		assert_eq!(
			headers.get("Content-Transfer-Encoding"),
			Some(&"BINARY".into()),
			"Content-Transfer-Encoding"
		);
		assert_eq!(headers.get("X-Binary-ID"), Some(&"1".into()), "X-Binary-ID");
		assert_eq!(
			headers.get("X-Binary-Element-Type"),
			Some(&"signed 32-bit integer".into()),
			"X-Binary-Element-Type"
		);
		assert_eq!(
			headers.get("X-Binary-Element-Byte-Order"),
			Some(&"LITTLE_ENDIAN".into()),
			"X-Binary-Element-Byte-Order"
		);
		assert_eq!(
			headers.get("X-Binary-Number-of-Elements"),
			Some(&"8294400".into()),
			"X-Binary-Number-of-Elements"
		);
		assert_eq!(
			headers.get("X-Binary-Size-Fastest-Dimension"),
			Some(&"2880".into()),
			"X-Binary-Size-Fastest-Dimension"
		);
		assert_eq!(
			headers.get("X-Binary-Size-Second-Dimension"),
			Some(&"2880".into()),
			"X-Binary-Size-Second-Dimension"
		);
		assert_eq!(
			headers.get("X-Binary-Size-Padding"),
			Some(&"1".into()),
			"X-Binary-Size-Padding"
		);
		assert_eq!(
			headers.get("Content-Type"),
			Some(&"application/octet-stream;conversions=\"x-CBF_BYTE_OFFSET\"".into()),
			"Content-Type"
		);
		assert_eq!(headers.get("X-Binary-Size"), Some(&"10161580".into()), "X-Binary-Size");
		assert_eq!(
			headers.get("Content-MD5"),
			Some(&"kL8G8UnwN1oKBdHWVkb0CQ==".into()),
			"Content-MD5"
		);
	}

	#[test]
	fn test_field_name() {
		assert_eq!(field_name("X-Binary-ID:"), Ok((":", "X-Binary-ID")));
		assert_eq!(field_name("Content-Type: "), Ok((": ", "Content-Type")));
		assert_eq!(field_name("X-Binary-Size : "), Ok((" : ", "X-Binary-Size")));
		assert_eq!(field_name("Content-MD5 :"), Ok((" :", "Content-MD5")));

		assert_eq!(field_name(":"), Err(Error(Error { input: ":", code: TakeWhile1 })));
		assert_eq!(field_name(": "), Err(Error(Error { input: ": ", code: TakeWhile1 })));
		assert_eq!(field_name(" : "), Err(Error(Error { input: " : ", code: TakeWhile1 })));
		assert_eq!(field_name(" :"), Err(Error(Error { input: " :", code: TakeWhile1 })));

		assert!(matches!(field_name(""), Err(Incomplete(Size(_)))));
	}

	#[test]
	fn test_field_body() {
		assert_eq!(
			field_body("text;\r\n    text\r\nNext-Field"),
			Ok(("\r\nNext-Field", "text;text".into()))
		);
		assert_eq!(
			field_body("text;\r\n\ttext\r\n\r\n"),
			Ok(("\r\n\r\n", "text;text".into()))
		);
	}

	#[test]
	fn test_field_body_contents() {
		assert_eq!(field_body_contents("\"q-string\"\r\n"), Ok(("\r\n", "q-string".into())));
		assert_eq!(
			field_body_contents("text text;\"=:text\r\n"),
			Ok(("\r\n", "text text;\"=:text".into()))
		);
		assert_eq!(field_body_contents("\"text\r\n"), Ok(("\r\n", "\"text".into())));
	}
}
