use nom::{bytes::complete::take_while1, number::complete::le_u32, sequence::tuple, IResult};

use crate::pe::make_parse_error;

use super::FileParseResult;

#[derive(Debug, PartialEq, Eq)]
struct DirectoryEntry {
    import_lookup_table_rva: u32,
    name_rva: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ImportedDll {
    pub name: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ImportTable {
    pub imports: Vec<ImportedDll>,
}

impl ImportTable {
    pub fn parse<'i>(
        input: &'i [u8],
        rva_to_file_slice: impl Fn(u32) -> Option<&'i [u8]>,
    ) -> FileParseResult<'i, Self> {
        let (remaining, directory_table) = ImportTable::parse_import_directory_table(input)?;

        let mut imports = Vec::new();
        for entry in &directory_table {
            // Jump to the rva
            let data = rva_to_file_slice(entry.name_rva).ok_or_else(|| make_parse_error(input))?;

            // Read the name
            let (_, name) = take_while1(|c| c != 0)(data)?;
            let name = std::str::from_utf8(name)
                .map_err(|_| make_parse_error(input))?
                .to_owned();

            imports.push(ImportedDll { name });
        }

        Ok((remaining, ImportTable { imports }))
    }

    fn parse_import_directory_table(mut input: &[u8]) -> IResult<&[u8], Vec<DirectoryEntry>> {
        let mut entries = vec![];
        loop {
            let (remaining, entry) = tuple((le_u32, le_u32, le_u32, le_u32, le_u32))(input)?;
            input = remaining;

            // Null entry, end of the table
            if entry.0 == 0 {
                break;
            }

            entries.push(DirectoryEntry {
                import_lookup_table_rva: entry.0,
                name_rva: entry.3,
            })
        }

        Ok((input, entries))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn import_directory_table() {
        let data = vec![
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b,
            0x1c, 0x1d, 0x1e, 0x1f, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];

        assert_eq!(
            ImportTable::parse_import_directory_table(&data).unwrap().1,
            vec![
                DirectoryEntry {
                    import_lookup_table_rva: 0x03020100,
                    name_rva: 0x0f0e0d0c,
                },
                DirectoryEntry {
                    import_lookup_table_rva: 0x17161514,
                    name_rva: 0x23222120,
                },
            ]
        );
    }
}
