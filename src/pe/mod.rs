mod coff_header;
mod file;
mod import_table;
mod msdos_header;
mod optional_header;
mod section_table;

pub use file::File;
use nom::error::ParseError;

#[derive(Debug, PartialEq, Eq)]
enum Architecture {
    X86,
    X64,
}

type FileParseResult<'i, T> = nom::IResult<&'i [u8], T>;

fn make_parse_error<T, E: ParseError<T>>(data: T) -> nom::Err<E> {
    nom::Err::Error(nom::error::make_error(data, nom::error::ErrorKind::Char))
}
