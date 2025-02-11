use crate::file_listing::file_entity::FileEntity;
use crate::file_listing::files::Files;
use crate::ntfs::file_record::FileRecord;
use rusqlite::Connection;
use rusqlite::Result;

const CREATE_DB: &str = "
    CREATE TABLE IF NOT EXISTS file_entry (
    _id           INTEGER PRIMARY KEY,
    id            INTEGER,
    parent_id     INTEGER,
    dos_flags     INTEGER,
    real_size     INTEGER,
    name          TEXT,
    modified_date INTEGER,
    created_date  INTEGER,
    flags         INTEGER,
    base_record   INTEGER,
    fr_number     INTEGER,
    namespace     INTEGER );
    ";
const INSERT_FILE: &str = "INSERT INTO file_entry (id, parent_id, dos_flags, real_size, name, modified_date, created_date, flags, base_record, fr_number, namespace) \
    VALUES (:id, :parent_id, :dos_flags, :real_size, :name, :modified_date, :created_date, :flags, :base_record, :fr_number, :namespace);";
const UPSERT_FILE: &str = "INSERT OR REPLACE INTO file_entry (id, parent_id, dos_flags, real_size, name, modified_date, created_date) \
    VALUES (:id, :parent_id, :dos_flags, :real_size, :name, :modified_date, :created_date);";
const UPDATE_FILE: &str = "UPDATE file_entry SET \
    id = :id, parent_id = :parent_id, dos_flags = :dos_flags, real_size = :real_size, name = :name, modified_date = :modified_date, created_date = :created_date \
    WHERE id = :id;";
const DELETE_FILE: &str = "DELETE FROM file_entry WHERE id = :id;";
const COUNT_FILES: &str = "SELECT COUNT(id) FROM file_entry where name like :name";
const SELECT_FILES: &str = "SELECT name, parent_id, real_size, id FROM file_entry where name like :name order by name limit :p_size;";
const SELECT_COUNT_ALL: &str = "SELECT COUNT(id) FROM file_entry;";
const SELECT_ALL_FILES: &str = "SELECT * FROM file_entry;";
const SELECT_FILES_NEXT_PAGE: &str = "SELECT name, parent_id, real_size, id FROM file_entry where name like :name and (name, id) >= (:p_name, :p_id) order by name limit :p_size;";
//const FILE_ENTRY_NAME_INDEX: &str = "CREATE INDEX IF NOT EXISTS file_entry_name ON file_entry(name, id);";

//const FILE_PAGE_SIZE: u32 = 3000;

pub fn main() -> Connection {
    let conn = Connection::open("test.db").unwrap();
    //    let conn = Connection::open_in_memory().unwrap();

    conn.execute(CREATE_DB, params![]).unwrap();
    conn.prepare_cached(INSERT_FILE).unwrap();
    conn.prepare_cached(UPDATE_FILE).unwrap();
    conn.prepare_cached(DELETE_FILE).unwrap();
    conn.prepare_cached(UPSERT_FILE).unwrap();
    conn.prepare_cached(COUNT_FILES).unwrap();
    conn.prepare_cached(SELECT_FILES).unwrap();
    conn.prepare_cached(SELECT_FILES_NEXT_PAGE).unwrap();
    conn
}

//pub fn delete_file(tx: &Transaction, file_id: u32) {
//    tx.execute_named(DELETE_FILE, &[
//        (":id", &file_id)]).unwrap();
//}

//pub fn upsert_file(tx: &Transaction, file: &FileRecord) {
//    tx.execute_named(UPSERT_FILE, &[
//        (":id", &file.id),
//        (":parent_id", &file.parent_id),
//        (":dos_flags", &file.dos_flags),
//        (":real_size", &file.real_size),
//        (":name", &file.name),
//        (":modified_date", &file.modified_date),
//        (":created_date", &file.created_date),
//    ]).unwrap();
//}

//pub fn update_file(tx: &Transaction, file: &FileRecord) {
//    tx.execute_named(UPDATE_FILE, &[
//        (":id", &file.id),
//        (":parent_id", &file.parent_id),
//        (":dos_flags", &file.dos_flags),
//        (":real_size", &file.real_size),
//        (":name", &file.name),
//        (":modified_date", &file.modified_date),
//        (":created_date", &file.created_date),
//    ]).unwrap();
//}

//pub fn create_indices(con: &Connection) {
//    con.execute(FILE_ENTRY_NAME_INDEX, &[]).unwrap();
//}

pub fn insert_files(files: &[FileRecord]) {
    let mut conn = main();
    let tx = conn.transaction().unwrap();
    {
        let mut stmt = tx.prepare_cached(INSERT_FILE).unwrap();
        for file in files {
            file.name_attrs
                .iter()
                .filter(|n| n.namespace != 2)
                .for_each(|name| {
                    stmt.execute_named(&[
                        (":id", &file.header.fr_number),
                        (":parent_id", &(name.parent_id as u32)),
                        (":dos_flags", &name.dos_flags),
                        (":real_size", &file.data_attr.size),
                        (":name", &name.name),
                        (":modified_date", &file.standard_attr.modified),
                        (":created_date", &file.standard_attr.created),
                        (":base_record", &(file.header.base_record as i64)),
                        (":fr_number", &file.fr_number()),
                        (":namespace", &name.namespace),
                        (":flags", &file.header.flags),
                    ])
                    .unwrap();
                });
        }
    }
    tx.commit().unwrap();
}

pub fn load_all_arena() -> Result<(Files)> {
    let con = Connection::open("test.db").unwrap();
    let count = con
        .query_row(SELECT_COUNT_ALL, params![], |r| r.get::<usize, u32>(0))
        .unwrap() as usize;
    let mut stmt = con.prepare(SELECT_ALL_FILES).unwrap();
    let result = stmt
        .query_map(params![], FileEntity::from_file_row)
        .unwrap();
    let mut files = Vec::with_capacity(count);
    for file in result {
        let f: FileEntity = file?;
        files.push(f);
    }
    let mut arena = Files::new(count);
    arena.bulk_add(files);
    Ok(arena)
}
