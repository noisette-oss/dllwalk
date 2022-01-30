use super::{
    coff_header::CoffHeader,
    import_table::{ImportTable, ImportedDll},
    make_parse_error,
    msdos_header::MsDosHeader,
    optional_header::OptionalHeader,
    section_table::SectionTable,
    FileParseResult,
};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct File {
    pub imports: Vec<ImportedDll>,
}

impl File {
    pub fn new() -> Self {
        Self {
            imports: Vec::new(),
        }
    }

    pub fn parse(data: &[u8]) -> FileParseResult<Self> {
        // MSDOS header
        let (_, msdos_header) = MsDosHeader::parse(data)?;

        // COFF header
        let (input, coff_header) = CoffHeader::parse(&data[msdos_header.pe_offset as usize..])?;

        // Optional header
        let (input, optional_header) = OptionalHeader::parse(input)?;

        // Section table
        let (_, section_table) = SectionTable::parse(input, coff_header.number_of_sections)?;

        // Imports
        let mut imports = Vec::new();
        if let Some(import_table_entry) = optional_header.get_import_table_entry() {
            if import_table_entry.rva != 0 {
                let import_table_offset = section_table
                    .rva_to_file_offset(import_table_entry.rva)
                    .ok_or_else(|| make_parse_error(input))?;

                let rva_to_file_slice = |rva| {
                    let offset = section_table.rva_to_file_offset(rva)?;
                    Some(&data[offset as usize..])
                };

                let (_, import_table) = ImportTable::parse(
                    &data[import_table_offset as usize..], 
                    rva_to_file_slice
                )?;

                imports = import_table.imports;
            }
        }

        Ok((data, File { imports }))
    }
}
