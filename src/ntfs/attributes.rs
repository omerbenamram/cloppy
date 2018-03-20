use std::io::Cursor;
use byteorder::{
    ByteOrder,
    LittleEndian,
    ReadBytesExt,
};
use std::ffi::OsString;
use std::os::windows::prelude::*;
use windows::utils::windows_string;

const DATARUN_END: u8 = 0x00;
const END1: u32 = 0xFFFFFFFF;
const STANDARD: u32 = 0x10;
pub const FILENAME: u32 = 0x30;
pub const DATA: u32 = 0x80;

#[derive(Debug, PartialEq)]
pub enum AttributeType {
    Standard(StandardAttr),
    Filename(FilenameAttr),
    Data(Vec<Datarun>),
}

#[derive(Debug, PartialEq)]
pub struct Attribute {
    pub attr_flags: u16,
    pub attr_type: AttributeType,
}

#[derive(Debug, PartialEq)]
pub struct StandardAttr {
    pub dos_flags: u32,
    pub modified: u64,
    pub created: u64,
}

#[derive(Debug, PartialEq)]
pub struct FilenameAttr {
    pub parent_id: u64,
    pub allocated_size: u64,
    pub real_size: u64,
    pub flags: u32,
    pub namespace: u8,
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub struct Datarun {
    pub length_lcn: u64,
    pub offset_lcn: i64,
}

fn length_in_lcn(input: &[u8]) -> u64 {
    let mut base: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
    for (i, b) in input.iter().take(8).enumerate() {
        base[i] = *b;
    }
    let mut rdr = Cursor::new(&base);
    rdr.read_u64::<LittleEndian>().unwrap()
}

fn offset_in_lcn(input: &[u8]) -> i64 {
    if *input.last().unwrap() < 0x80 {
        length_in_lcn(&input) as i64
    } else {
        let two_comp = input.iter().map(|b| !(*b) as i8 as u8).collect::<Vec<u8>>();
        -(length_in_lcn(&two_comp) as i64) - 1
    }
}

fn data_attr(input: &[u8]) -> Vec<Datarun> {
    let mut offset = 0;
    let mut dataruns = vec![];
    loop {
        if input[offset] == DATARUN_END {
            break;
        }
        let header = input[offset];
        offset += 1;
        let offset_size = (header >> 4) as usize;
        let length_size = (header & 0x0F) as usize;
        let length_lcn = length_in_lcn(&input[offset..offset + length_size]);
        offset += length_size;
        let offset_lcn = offset_in_lcn(&input[offset..offset + offset_size]);
        dataruns.push(Datarun { length_lcn, offset_lcn });
        offset += offset_size;
    }
    dataruns
}

fn filename_attr(input: &[u8]) -> FilenameAttr {
    let parent_id = LittleEndian::read_u64(input);
    let allocated_size = LittleEndian::read_u64(&input[0x28..]);
    let real_size = LittleEndian::read_u64(&input[0x30..]);
    let flags = LittleEndian::read_u32(&input[0x38..]);
    let name_length = (input[0x40] as u16 * 2) as usize;
    let namespace = input[0x41];
    let name = &input[0x42..0x42 + name_length];
    FilenameAttr {
        parent_id,
        allocated_size,
        real_size,
        namespace,
        flags,
        name: windows_string(name),
    }
}

fn standard_attr(input: &[u8]) -> StandardAttr {
    let created = LittleEndian::read_u64(input);
    let modified = LittleEndian::read_u64(&input[0x08..]);
    let dos_flags = LittleEndian::read_u32(&input[0x20..]);
    StandardAttr { modified, created, dos_flags }
}

pub fn parse_attributes(input: &[u8], last_attr: u32) -> Vec<Attribute> {
    let mut parsed_attributes: Vec<Attribute> = Vec::with_capacity(2);
    let mut offset = 0;
    loop {
        let attr_type = LittleEndian::read_u32(&input[offset..]);
        if attr_type == END1 || attr_type > last_attr {
            break;
        }
        let attr_flags = LittleEndian::read_u16(&input[offset + 0x0C..]);
        let attr_length = LittleEndian::read_u32(&input[offset + 0x04..]) as usize;
        if attr_type == STANDARD || attr_type == FILENAME {
            let attr_offset = LittleEndian::read_u16(&input[offset + 0x14..]) as usize;
            if attr_type == STANDARD {
                let standard = standard_attr(&input[offset + attr_offset..]);
                parsed_attributes.push(Attribute {
                    attr_flags,
                    attr_type: AttributeType::Standard(standard),
                });
            } else {
                let filename = filename_attr(&input[offset + attr_offset..]);
                parsed_attributes.push(Attribute {
                    attr_flags,
                    attr_type: AttributeType::Filename(filename),
                });
            }
        } else if attr_type == DATA {
            let attr_offset = LittleEndian::read_u16(&input[offset + 0x20..]) as usize;
            let data = data_attr(&input[offset + attr_offset..]);
            parsed_attributes.push(Attribute {
                attr_flags,
                attr_type: AttributeType::Data(data),
            });
        }
        offset += attr_length;
    }
    parsed_attributes
}

#[cfg(test)]
mod tests {
    use super::*;
    use ntfs::attributes::AttributeType::*;

    #[test]
    fn test_length_in_lcn() {
        assert_eq!(0xAA, length_in_lcn(&[0xAA]));
        assert_eq!(0xAABBCCDD11223344, length_in_lcn(&[0x44, 0x33, 0x22, 0x11, 0xDD, 0xCC, 0xBB, 0xAA]));
        assert_eq!(0xAABBCCDD11223344, length_in_lcn(&[0x44, 0x33, 0x22, 0x11, 0xDD, 0xCC, 0xBB, 0xAA, 0xFF]));
    }

    #[test]
    fn test_positive_offset_in_lcn() {
        assert_eq!(0x77, offset_in_lcn(&[0x77]));
        assert_eq!(0x7777777777777755, offset_in_lcn(&[0x55, 0x77, 0x77, 0x77, 0x77, 0x77, 0x77, 0x77]));
        assert_eq!(0x7777777777777755, offset_in_lcn(&[0x55, 0x77, 0x77, 0x77, 0x77, 0x77, 0x77, 0x77, 0xFF]));
    }

    #[test]
    fn test_negative_offset_in_lcn() {
        assert_eq!(-0x80, offset_in_lcn(&[0x80]));
        assert_eq!(-0xE44B1, offset_in_lcn(&[0x4F, 0xBB, 0xF1]));
        assert_eq!(-0xFF55DE, offset_in_lcn(&[0x22, 0xAA, 0x00, 0xFF]));
    }

    #[test]
    fn test_data_attr() {
        let input = [51, 32, 200, 0, 0, 0, 12, 67, 236, 207, 0, 118, 65, 153, 0, 67, 237, 201, 0, 94, 217, 243, 0, 51, 72, 235, 0, 12, 153, 121, 67, 191, 6, 5, 60, 11, 224, 0, 0];
        let output = [
            Datarun { length_lcn: 51232, offset_lcn: 786432 },
            Datarun { length_lcn: 53228, offset_lcn: 10043766 },
            Datarun { length_lcn: 51693, offset_lcn: 15980894 },
            Datarun { length_lcn: 60232, offset_lcn: 7969036 },
            Datarun { length_lcn: 329407, offset_lcn: 14682940 }];
        assert_eq!(&output, data_attr(&input).as_slice());
    }

    #[test]
    fn test_filename_attr() {
        let input = [5, 0, 0, 0, 0, 0, 5, 0, 82, 131, 14, 254, 172, 15, 209, 1, 82, 131, 14, 254, 172, 15, 209, 1, 82, 131, 14, 254, 172, 15, 209, 1, 82, 131, 14, 254, 172, 15, 209, 1, 0, 64, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 4, 3, 36, 0, 77, 0, 70, 0, 84, 0];
        let output = FilenameAttr { parent_id: 1407374883553285, allocated_size: 16384, real_size: 16384, flags: 6, namespace: 3, name: "$MFT".to_string() };
        assert_eq!(output, filename_attr(&input));
    }

    #[test]
    fn test_standard_attr() {
        let input = [82, 131, 14, 254, 172, 15, 209, 1, 82, 131, 14, 254, 172, 15, 209, 1, 82, 131, 14, 254, 172, 15, 209, 1, 82, 131, 14, 254, 172, 15, 209, 1, 6, 0, 0, 0];
        let output = StandardAttr { dos_flags: 6, modified: 130903099841610578, created: 130903099841610578 };
        assert_eq!(output, standard_attr(&input));
    }

    #[test]
    fn test_parse_attributes() {
        let output = vec![Attribute { attr_flags: 0, attr_type: Standard(StandardAttr { dos_flags: 6, modified: 130903099841610578, created: 130903099841610578 }) }, Attribute { attr_flags: 0, attr_type: Filename(FilenameAttr { parent_id: 1407374883553285, allocated_size: 16384, real_size: 16384, flags: 6, namespace: 3, name: "$MFT".to_string() }) }, Attribute { attr_flags: 0, attr_type: Data(vec![Datarun { length_lcn: 51232, offset_lcn: 786432 }, Datarun { length_lcn: 53228, offset_lcn: 10043766 }, Datarun { length_lcn: 51693, offset_lcn: 15980894 }, Datarun { length_lcn: 60232, offset_lcn: 7969036 }, Datarun { length_lcn: 329407, offset_lcn: 14682940 }]) }];
        let input = [16, 0, 0, 0, 96, 0, 0, 0, 0, 0, 24, 0, 0, 0, 0, 0, 72, 0, 0, 0, 24, 0, 0, 0, 82, 131, 14, 254, 172, 15, 209, 1, 82, 131, 14, 254, 172, 15, 209, 1, 82, 131, 14, 254, 172, 15, 209, 1, 82, 131, 14, 254, 172, 15, 209, 1, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 48, 0, 0, 0, 104, 0, 0, 0, 0, 0, 24, 0, 0, 0, 3, 0, 74, 0, 0, 0, 24, 0, 1, 0, 5, 0, 0, 0, 0, 0, 5, 0, 82, 131, 14, 254, 172, 15, 209, 1, 82, 131, 14, 254, 172, 15, 209, 1, 82, 131, 14, 254, 172, 15, 209, 1, 82, 131, 14, 254, 172, 15, 209, 1, 0, 64, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 4, 3, 36, 0, 77, 0, 70, 0, 84, 0, 0, 0, 0, 0, 0, 0, 128, 0, 0, 0, 104, 0, 0, 0, 1, 0, 64, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 83, 8, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 133, 0, 0, 0, 0, 0, 0, 64, 133, 0, 0, 0, 0, 0, 0, 64, 133, 0, 0, 0, 0, 51, 32, 200, 0, 0, 0, 12, 67, 236, 207, 0, 118, 65, 153, 0, 67, 237, 201, 0, 94, 217, 243, 0, 51, 72, 235, 0, 12, 153, 121, 67, 191, 6, 5, 60, 11, 224, 0, 0, 0, 176, 0, 0, 0];
        assert_eq!(output, parse_attributes(&input, DATA));
    }
}

