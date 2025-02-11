use crate::errors::MyErrorKind::UsnJournalError;
use crate::ntfs::change_journal::usn_record::UsnChange;
use crate::ntfs::change_journal::usn_record::UsnRecord;
use crate::ntfs::file_record::FileRecord;
use crate::ntfs::volume_data::VolumeData;
use crate::ntfs::windows_api::get_file_record;
use crate::ntfs::windows_api::get_usn_journal;
use crate::ntfs::windows_api::get_volume_data;
use crate::ntfs::windows_api::read_usn_journal;
use crate::ntfs::windows_api::UsnJournal as WinJournal;
use byteorder::{ByteOrder, LittleEndian};
use failure::{Error, ResultExt};
use std::fs::File;
use std::mem;
use std::path::Path;
use winapi::shared::minwindef::BYTE;
use winapi::um::winioctl::NTFS_FILE_RECORD_OUTPUT_BUFFER;

pub struct UsnJournal {
    volume: File,
    volume_data: VolumeData,
    usn_journal_id: u64,
    next_usn: i64,
}

impl UsnJournal {
    pub fn new<P: AsRef<Path>>(volume_path: P) -> Result<Self, Error> {
        let volume = File::open(volume_path).context(UsnJournalError)?;
        let volume_data = get_volume_data(&volume)
            .map(VolumeData::new)
            .context(UsnJournalError)?;
        let WinJournal {
            usn_journal_id,
            next_usn,
        } = get_usn_journal(&volume).context(UsnJournalError)?;
        Ok(UsnJournal {
            volume,
            volume_data,
            usn_journal_id,
            next_usn,
        })
    }

    pub fn get_new_changes(&mut self) -> Result<Vec<UsnChange>, Error> {
        let mut buffer = vec![0u8; self.volume_data.bytes_per_cluster as usize];
        let mut output_buffer =
            [0u8; mem::size_of::<NTFS_FILE_RECORD_OUTPUT_BUFFER>() + mem::size_of::<BYTE>() * 4096];
        let buffer = read_usn_journal(
            &self.volume,
            self.next_usn,
            self.usn_journal_id,
            &mut buffer,
        )
        .context(UsnJournalError)?;
        let mut usn_records = vec![];
        let next_usn = LittleEndian::read_i64(buffer);
        let mut offset = 8;
        loop {
            if offset == buffer.len() {
                break;
            }
            let record = UsnRecord::new(&buffer[offset..]).context(UsnJournalError)?;
            offset += record.length;

            let fr_buffer =
                get_file_record(&self.volume, record.fr_number, &mut output_buffer).unwrap();
            let entry = FileRecord::parse_mft_entry(fr_buffer, self.volume_data);
            if let Some(f) = entry {
                usn_records.push(record.into_change(f))
            }
        }
        self.next_usn = next_usn;
        Ok(usn_records)
    }
}
