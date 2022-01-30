use nom::{
    bytes::complete::{tag, take},
    number::complete::le_u16,
    sequence::tuple,
};

use super::FileParseResult;

#[derive(Debug, PartialEq, Eq)]
pub struct CoffHeader {
    pub number_of_sections: u16,
    pub size_of_optional_header: u16,
}

impl CoffHeader {
    pub fn parse(input: &[u8]) -> FileParseResult<Self> {
        let (input, (_, _, number_of_sections, _, size_of_optional_header, _)) = tuple((
            tag("PE\0\0".as_bytes()),
            le_u16,
            le_u16,
            take(12_usize),
            le_u16,
            le_u16,
        ))(input)?;

        Ok((
            input,
            CoffHeader {
                number_of_sections,
                size_of_optional_header,
            },
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn coff_header() {
        let data = vec![
            0x50, 0x45, 0x00, 0x00, 0x00, 0x00, 0x02, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x34, 0x12, 0x00, 0x00,
        ];

        assert_eq!(
            CoffHeader::parse(&data).unwrap().1,
            CoffHeader {
                number_of_sections: 0x0102,
                size_of_optional_header: 0x1234
            }
        );

        assert_eq!(CoffHeader::parse(&vec![0u8; 100]).is_err(), true);
    }
}
