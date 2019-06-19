pub mod attributes;
pub mod change_journal;
pub mod file_record;
mod mft_parser;
mod mft_reader;
pub mod parse_operation;
mod volume_data;
mod windows_api;

//TODO make this value 'smart' depending on the HD
const FR_AT_ONCE: u64 = 4 * 16;
