use crate::ntfs::change_journal::UsnRecord;
use crate::ntfs::file_record::FileRecord;
use rusqlite::Result;
use rusqlite::Row;
use std::u32;

const DOS_NAMESPACE: u8 = 2;

#[derive(Clone, Debug, PartialEq)]
pub struct FileEntity {
    name: String,
    parent_id: FileId,
    size: i64,
    id: FileId,
    _id: u32,
    flags: u16,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialOrd, PartialEq, Hash)]
pub struct FileId {
    id: u32,
    f_type: FileType,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum FileType {
    DIRECTORY,
    FILE,
}

impl From<UsnRecord> for FileId {
    fn from(record: UsnRecord) -> Self {
        if record.is_dir() {
            FileId::directory(record.mft_id)
        } else {
            FileId::file(record.mft_id)
        }
    }
}

impl FileId {
    pub fn file(id: u32) -> FileId {
        FileId {
            id,
            f_type: FileType::FILE,
        }
    }
    pub fn directory(id: u32) -> FileId {
        FileId {
            id,
            f_type: FileType::DIRECTORY,
        }
    }
    pub fn id(self) -> u32 {
        self.id
    }

    pub fn f_type(self) -> FileType {
        self.f_type
    }
}

impl From<FileRecord> for FileEntity {
    fn from(file: FileRecord) -> Self {
        let fr_number = file.fr_number();
        let name = file
            .name_attrs
            .into_iter()
            .filter(|n| n.namespace != DOS_NAMESPACE)
            .take(1)
            .next()
            .unwrap_or_else(|| panic!("Found a file record without name: {}", fr_number));

        let id = if file.header.flags & 0x02 != 0 {
            FileId::directory(file.header.fr_number)
        } else {
            FileId::file(file.header.fr_number)
        };
        FileEntity {
            name: name.name,
            parent_id: FileId::directory(name.parent_id as u32),
            size: file.data_attr.size,
            id,
            _id: u32::MAX,
            flags: file.header.flags,
        }
    }
}

impl FileEntity {
    pub fn from_file_row(row: &Row) -> Result<FileEntity> {
        let _id = row.get::<usize, u32>(0)?;
        let parent_id = FileId::directory(row.get::<usize, i64>(2)? as u32);
        let size = row.get::<usize, i64>(4)?;
        let name = row.get::<usize, String>(5)?;
        let flags = row.get::<usize, u16>(8)?;
        let id = if flags & 0x02 != 0 {
            FileId::directory(row.get::<usize, u32>(1)?)
        } else {
            FileId::file(row.get::<usize, u32>(1)?)
        };
        Ok(FileEntity {
            name,
            parent_id,
            size,
            id,
            _id,
            flags,
        })
    }

    pub fn id(&self) -> FileId {
        self.id
    }

    pub fn parent_id(&self) -> FileId {
        self.parent_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn size(&self) -> i64 {
        self.size
    }

    pub fn flags(&self) -> u16 {
        self.flags
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usn_record_to_file_id_file() {
        let mut record = UsnRecord::default();
        record.mft_id = 99;
        record.flags = 0x30;
        assert_eq!(FileId::file(99), record.into());
    }

    #[test]
    fn usn_record_to_file_id_dir() {
        let mut record = UsnRecord::default();
        record.mft_id = 99;
        record.flags = 0x16;
        assert_eq!(FileId::directory(99), record.into());
    }
}
