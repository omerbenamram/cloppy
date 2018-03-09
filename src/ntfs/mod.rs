use self::attributes::*;
use self::volume_data::VolumeData;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;
use windows;
use byteorder::{
    ByteOrder,
    LittleEndian,
};

mod volume_data;
mod file_entry;
mod attributes;

const END1: u32 = 0xFFFFFFFF;
const STANDARD: u32 = 0x10;
const FILENAME: u32 = 0x30;
const DATA: u32 = 0x80;
const DATARUN_END: u8 = 0x00;

pub struct MftParser {
    file: File,
    volume_data: VolumeData,
    buffer1: Vec<u8>,
    buffer: [u8; SPEED_FACTOR as usize * 1024],
    count: u64,
}

//TODO make this value 'smart' depending on the HD
const SPEED_FACTOR: u64 = 4 * 16;

impl MftParser {
    //    pub fn new<P: AsRef<Path>>(volume_path: P) -> Self {
    pub fn new(volume_path: &str) -> Self {
//        let file = File::open(volume_path).expect("Failed to open volume handle");
        let file = windows::open_file(volume_path);
        let volume_data = VolumeData::new(windows::open_volume(&file));
        let buffer1 = Vec::with_capacity(SPEED_FACTOR as usize * volume_data.bytes_per_file_record as usize);
        let buffer = [0; SPEED_FACTOR as usize * 1024];
//        let buffer = vec![0; SPEED_FACTOR as usize * volume_data.bytes_per_cluster as usize];
        MftParser { file, volume_data, buffer1, buffer, count: 0 }
    }

    fn parse_chunk(&mut self, offset: u64, chunk_number: u64, size: usize) {
        let from = SeekFrom::Start(offset + SPEED_FACTOR * chunk_number * self.volume_data.bytes_per_file_record as u64);
        self.fill_buffer(from);
        for buff in self.buffer.chunks_mut(self.volume_data.bytes_per_file_record as usize).take(size) {
            MftParser::read_file_record(buff, &self.volume_data);
            self.count += 1;
        }
    }

    pub fn parse(&mut self, fr0: file_entry::FileEntry) {
        //        let fr0 = self.read_mft0();
//        println!("{:#?}", fr0);
        use std::time::Instant;
        let mut absolute_lcn_offset = 0i64;
        let now = Instant::now();
        for (i, run) in fr0.dataruns.iter().enumerate() {
            absolute_lcn_offset += run.offset_lcn;
            let absolute_offset = absolute_lcn_offset as u64 * self.volume_data.bytes_per_cluster as u64;
            let mut file_record_count = run.length_lcn * self.volume_data.clusters_per_fr() as u64;
//            let mut file_record_count = 2048;
            println!("datarun {} started", file_record_count);

            let full_runs = file_record_count / SPEED_FACTOR;
            let partial_run_size = file_record_count % SPEED_FACTOR;
            for run in 0..full_runs {
                self.parse_chunk(absolute_offset, run, SPEED_FACTOR as usize);
            }
            self.parse_chunk(absolute_offset, full_runs - 1, partial_run_size as usize);
            println!("datarun {} finished", i);
            println!("total time {:?}", Instant::now().duration_since(now));
            println!("total files {:?}", self.count);
        }
    }

    pub fn read_mft0(&mut self) -> file_entry::FileEntry {
        let from = SeekFrom::Start(self.volume_data.initial_offset());
        self.fill_buffer(from);
        MftParser::read_file_record0(&mut self.buffer[0..self.volume_data.bytes_per_file_record as usize], &self.volume_data)
    }

    fn fill_buffer(&mut self, offset: SeekFrom) {
        self.file.seek(offset).unwrap();
        let buffer = &mut self.buffer;
        let file = &mut self.file;
        let x = Vec::<u32>::with_capacity(buffer.len());
        if x.capacity() == 0 {
            panic!();
        }
        windows::read_file(file, buffer).unwrap();
//        file.read_exact(buffer).unwrap();
    }
    fn read_file_record0(buffer: &mut [u8], volume_data: &VolumeData) -> file_entry::FileEntry {
        match file_record_header(buffer) {
            Some(header) => {
                let frn = header.fr_number;
                for (i, chunk) in header.fixup_seq.chunks(2).skip(1).enumerate() {
                    buffer[volume_data.bytes_per_sector as usize * (i + 1) - 2] = *chunk.first().unwrap();
                    buffer[volume_data.bytes_per_sector as usize * (i + 1) - 1] = *chunk.last().unwrap();
                }
                let attributes = parse_attributes1(&buffer[header.attr_offset as usize..], DATA);
                file_entry::FileEntry::new(attributes, frn)
//                match parse_attributes(attr_parser, &buffer[header.attr_offset as usize..]) {
//                    IResult::Done(_, r) => {
//                        let entry = file_entry::FileEntry::new(r.0, frn);
//                        return entry;
//                    }
//                    _ => {
//                        println!("error or incomplete");
//                        panic!("cannot parse attributes");
//                    }
//                }
            }
            _ => return file_entry::FileEntry::default()
        }
    }
    fn read_file_record(buffer: &mut [u8], volume_data: &VolumeData) -> file_entry::FileEntry {
        let result = match file_record_header(buffer) {
            Some(header) => {
                for (i, chunk) in header.fixup_seq.chunks(2).skip(1).enumerate() {
                    buffer[volume_data.bytes_per_sector as usize * (i + 1) - 2] = *chunk.first().unwrap();
                    buffer[volume_data.bytes_per_sector as usize * (i + 1) - 1] = *chunk.last().unwrap();
                }
                parse_attributes1(&buffer[header.attr_offset..], FILENAME);
                file_entry::FileEntry::default()
            }
            None => file_entry::FileEntry::default()
        };
        result
//        match file_record_header(buffer) {
//            Some((frn, header)) => {
//                for (i, chunk) in header.fixup_seq.chunks(2).skip(1).enumerate() {
//                    buffer[volume_data.bytes_per_sector as usize * (i + 1) - 2] = *chunk.first().unwrap();
//                    buffer[volume_data.bytes_per_sector as usize * (i + 1) - 1] = *chunk.last().unwrap();
//                }
//                match parse_attributes(attr_parser, &buffer[header.attr_offset as usize..]) {
//                    IResult::Done(_, r) => {
//                        let entry = file_entry::FileEntry::new(r.0, frn);
//                        return entry;
//                    }
//                    _ => {
//                        println!("error or incomplete");
//                        panic!("cannot parse attributes");
//                    }
//                }
//            }
//            _ => return file_entry::FileEntry::default()
//        }
//        file_entry::FileEntry::default()
    }
}

#[derive(Debug)]
struct FileRecordHeader {
    fr_number: u32,
    fixup_seq: Vec<u8>,
    attr_offset: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    const FR0: [u8; 1024] = [70, 73, 76, 69, 48, 0, 3, 0, 80, 245, 122, 254, 24, 0, 0, 0, 1, 0, 1, 0, 56, 0, 1, 0, 184, 1, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 99, 6, 255, 255, 0, 0, 0, 0, 16, 0, 0, 0, 96, 0, 0, 0, 0, 0, 24, 0, 0, 0, 0, 0, 72, 0, 0, 0, 24, 0, 0, 0, 82, 131, 14, 254, 172, 15, 209, 1, 82, 131, 14, 254, 172, 15, 209, 1, 82, 131, 14, 254, 172, 15, 209, 1, 82, 131, 14, 254, 172, 15, 209, 1, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 48, 0, 0, 0, 104, 0, 0, 0, 0, 0, 24, 0, 0, 0, 3, 0, 74, 0, 0, 0, 24, 0, 1, 0, 5, 0, 0, 0, 0, 0, 5, 0, 82, 131, 14, 254, 172, 15, 209, 1, 82, 131, 14, 254, 172, 15, 209, 1, 82, 131, 14, 254, 172, 15, 209, 1, 82, 131, 14, 254, 172, 15, 209, 1, 0, 64, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 4, 3, 36, 0, 77, 0, 70, 0, 84, 0, 0, 0, 0, 0, 0, 0, 128, 0, 0, 0, 104, 0, 0, 0, 1, 0, 64, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 83, 8, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 133, 0, 0, 0, 0, 0, 0, 64, 133, 0, 0, 0, 0, 0, 0, 64, 133, 0, 0, 0, 0, 51, 32, 200, 0, 0, 0, 12, 67, 236, 207, 0, 118, 65, 153, 0, 67, 237, 201, 0, 94, 217, 243, 0, 51, 72, 235, 0, 12, 153, 121, 67, 191, 6, 5, 60, 11, 224, 0, 0, 0, 176, 0, 0, 0, 72, 0, 0, 0, 1, 0, 64, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 66, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 0, 48, 4, 0, 0, 0, 0, 0, 0, 48, 4, 0, 0, 0, 0, 0, 0, 48, 4, 0, 0, 0, 0, 0, 49, 67, 118, 24, 3, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0, 0, 94, 177, 15, 1, 224, 99, 6, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 99, 6];

    #[test]
    fn test_withoutnom1() {
        let x = file_record_header1(&FR0);
        println!("{:?}", x);
    }

    #[test]
    fn test_withnom1() {
        let x = file_record_header(&FR0);
        println!("{:?}", x);
    }

    #[bench]
    fn bench_withoutnom(b: &mut Bencher) {
        b.iter(|| file_record_header1(&FR0));
    }

    #[bench]
    fn bench_withnom(b: &mut Bencher) {
        b.iter(|| file_record_header(&FR0));
    }
}

fn file_record_header(input: &[u8]) -> Option<FileRecordHeader> {
    if input[..4] == b"FILE"[..] {
        let fixup_offset = LittleEndian::read_u16(&input[0x4..]) as usize;
        let fixup_size = LittleEndian::read_u16(&input[0x06..]) as usize;
        let attr_offset = LittleEndian::read_u16(&input[0x14..]) as usize;
        let fr_number = LittleEndian::read_u32(&input[0x2C..]);
        let fixup_seq = input[fixup_offset..fixup_offset + 2 * fixup_size].to_vec();
        Some(FileRecordHeader {
            fr_number,
            attr_offset,
            fixup_seq,
        })
    } else {
        None
    }
}

fn parse_attributes1(input: &[u8], last_attr: u32) -> Vec<Attribute> {
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
                let standard = standard_attr1(&input[offset + attr_offset..]);
                parsed_attributes.push(Attribute {
                    attr_flags,
                    attr_type: AttributeType::Standard(standard),
                });
            } else {
                let filename = filename_attr1(&input[offset + attr_offset..]);
                parsed_attributes.push(Attribute {
                    attr_flags,
                    attr_type: AttributeType::Filename(filename),
                });
            }
        } else if attr_type == DATA {
            let attr_offset = LittleEndian::read_u16(&input[offset + 0x20..]) as usize;
            let data = data_attr1(&input[offset + attr_offset..]);
            parsed_attributes.push(Attribute {
                attr_flags,
                attr_type: AttributeType::Data(data),
            });
        }
        offset += attr_length;
    }
    parsed_attributes
}

fn data_attr1(input: &[u8]) -> Vec<Datarun> {
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

fn filename_attr1(input: &[u8]) -> FilenameAttr {
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

fn standard_attr1(input: &[u8]) -> StandardAttr {
    let created = LittleEndian::read_u64(input);
    let modified = LittleEndian::read_u64(&input[0x08..]);
    let dos_flags = LittleEndian::read_u32(&input[0x20..]);
    StandardAttr { modified, created, dos_flags }
}

