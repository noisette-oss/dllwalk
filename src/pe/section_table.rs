use nom::{
    bytes::complete::take,
    multi::count,
    number::complete::{le_u16, le_u32},
    sequence::tuple,
};

use super::FileParseResult;

#[derive(Debug, PartialEq, Eq)]
pub struct Section {
    name: String,
    virtual_size: u32,
    virtual_address: u32,
    raw_data_size: u32,
    raw_data_address: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SectionTable {
    sections: Vec<Section>,
}

impl SectionTable {
    pub fn parse(input: &[u8], number_of_sections: u16) -> FileParseResult<Self> {
        let (input, data) = count(
            tuple((
                take(8_usize),
                le_u32,
                le_u32,
                le_u32,
                le_u32,
                le_u32,
                le_u32,
                le_u16,
                le_u16,
                le_u32,
            )),
            number_of_sections as usize,
        )(input)?;

        let sections = data
            .iter()
            .map(|data| Section {
                name: String::from_utf8_lossy(data.0)
                    .trim_end_matches(|c| c == '\0')
                    .to_owned(),
                virtual_size: data.1,
                virtual_address: data.2,
                raw_data_size: data.3,
                raw_data_address: data.4,
            })
            .collect();

        Ok((input, SectionTable { sections }))
    }

    pub fn rva_to_file_offset(&self, rva: u32) -> Option<u32> {
        for section in &self.sections {
            if section.virtual_address <= rva
                && rva < section.virtual_address + section.virtual_size
            {
                return Some(section.raw_data_address + rva - section.virtual_address);
            }
        }

        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn section_table() {
        // x86
        let data = vec![
            0x2e, 0x69, 0x64, 0x61, 0x74, 0x61, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05,
            0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x61, 0x61,
            0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
            0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        assert_eq!(
            SectionTable::parse(&data, 2).unwrap().1,
            SectionTable {
                sections: vec![
                    Section {
                        name: ".idata".to_owned(),
                        virtual_size: 0x03020100,
                        virtual_address: 0x07060504,
                        raw_data_size: 0x0b0a0908,
                        raw_data_address: 0x0f0e0d0c,
                    },
                    Section {
                        name: "aaaaaaaa".to_owned(),
                        virtual_size: 0x13121110,
                        virtual_address: 0x17161514,
                        raw_data_size: 0x1b1a1918,
                        raw_data_address: 0x1f1e1d1c,
                    },
                ],
            }
        );
    }

    #[test]
    fn rva_to_raw() {
        let section_table = SectionTable {
            sections: vec![
                Section {
                    name: "".to_owned(),
                    virtual_size: 0x100,
                    virtual_address: 0x1000,
                    raw_data_size: 0x100,
                    raw_data_address: 0x500,
                },
                Section {
                    name: "".to_owned(),
                    virtual_size: 0x100,
                    virtual_address: 0x2000,
                    raw_data_size: 0x100,
                    raw_data_address: 0x800,
                },
            ],
        };

        assert_eq!(section_table.rva_to_file_offset(0x1010), Some(0x510));
        assert_eq!(section_table.rva_to_file_offset(0x2080), Some(0x880));
        assert_eq!(section_table.rva_to_file_offset(0x0fff), None);
        assert_eq!(section_table.rva_to_file_offset(0x1100), None);
        assert_eq!(section_table.rva_to_file_offset(0x1fff), None);
        assert_eq!(section_table.rva_to_file_offset(0x2100), None);
    }
}
