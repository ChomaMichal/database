pub mod file;
use file::File;

pub const PAGE: usize = 8096;
/* create table system.
* one file is one table
*
* file is separated to pages
* pages are const size
*
* there will be cells
* one cell is a row in the table
*
* each cell will also have a header that will point to begining of different colummns in this row
* if cell can't fit to the page the cell header will contain index of page whiere rest of the cell
* is located
*
* there can be multiple cells per page
*
* figure out resizing of cells
*
* how to implement btree
* use the first page for  metadata
* figure out how to handle collum insertions without copying the whole table or how it might be OK
*/

struct FileString {
    next: u64,      //where the string might continue
    string: String, // the string
}

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum ColTypes {
    Empty = 0,
    I32 = 1,
    U32 = 2,
    I64 = 3,
    U64 = 4,
    String = 5,
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum PageType {
    Empty = 0,         //page is currently empty
    DataStructure = 1, //pagee containts datastructure node
    Data = 2,          // page contains data
    MetaData = 3,      // page contains data about the table
}

//first page always contains metadata ther will be pointerst to all columns and indexes
#[repr(u8)]
enum MetaDataTypes {
    End = 0,    //metadata ended
    Column = 1, //len of the next column
    Index = 2,  //len of next index (index is for example the btree and key based on which it is
    //created)
    NextPage = 3, //index of next page with metadata
    FreePage = 4, //gives index of page that isn't used
}

struct FreePage {
    page_index: u64,
}

struct Indexing {
    name: String,    //name by which is the index created
    page_index: u64, // which page contains the head of index
}

pub struct Column {
    pub col_type: ColTypes,
    pub name: String,
}

struct DataHeader {
    offset: u64, //how far from index does it begin
    len: u64,    //how long is it
    cont: u64,   //if it continues on another page is 0 in case it is only in one page
}

struct PageHeader {
    page_type: PageType,
}

pub struct Table {
    file: File,
    indexes: Vec<Indexing>,
    columns: Vec<Column>,
    free_page: FreePage,
}

fn col_type_to_str(tp: &ColTypes) -> String {
    match tp {
        ColTypes::Empty => "Empty".to_owned(),
        ColTypes::I32 => "i32".to_owned(),
        ColTypes::U32 => "u32".to_owned(),
        ColTypes::I64 => "i64".to_owned(),
        ColTypes::U64 => "u64".to_owned(),
        ColTypes::String => "String".to_owned(),
    }
}

impl Table {
    fn new(file: File) -> Self {
        Self {
            file,
            indexes: vec![],
            columns: vec![],
            free_page: FreePage { page_index: 1 },
        }
    }

    pub fn display(&self) {
        let mut len = 1;
        for it in self.columns.iter() {
            len += it.name.len() + 6 + col_type_to_str(&it.col_type).len();
        }
        for _ in 0..len {
            print!("-");
        }
        println!("");
        print!("|");
        for it in self.columns.iter() {
            print!(" {} : {} |", it.name, col_type_to_str(&it.col_type));
        }
        println!("");
        for _ in 0..len {
            print!("-");
        }
    }
    //call before returning new table
    fn init_metadata(&mut self) -> Option<()> {
        let mut buf = [0u8; PAGE];
        let mut index = 0;
        let mut current_page = 0;
        self.file.new_page();
        buf[0] = PageType::MetaData as u8;
        index += 1;
        for it in self.columns.iter() {
            let len = it.name.as_bytes().len() + 13; // 1 because of the type 4 becasue of the
            // len fo the index 4 because of the len fo the string 4 because of the head of indexand len of the string
            if index + len + 9 > PAGE {
                // i think it should be + 5 becasue one of type4 of the
                // index
                let new_page = self.file.new_page().unwrap();
                serialize_metadata_next_page(&mut buf, &mut index, new_page);
                self.file.write_to_page(current_page, &mut buf).unwrap();
                index = 0;
                current_page = new_page;
            }
            serialize_metadata_column(&mut buf, &mut index, it);
        }

        for it in self.indexes.iter() {
            let len = it.name.as_bytes().len() + 13; // 1 because of the type 4 becasue of the
            // len fo the index 4 because of the len fo the string 4 because of the head of indexand len of the string
            if index + len + 9 > PAGE {
                // i think it should be + 5 becasue one of type4 of the
                // index
                let new_page = self.file.new_page().unwrap();
                serialize_metadata_next_page(&mut buf, &mut index, new_page);
                self.file.write_to_page(current_page, &mut buf).unwrap();
                index = 0;
                current_page = new_page;
            }
            serialize_metadata_index(&mut buf, &mut index, it);
        }
        serialize_metadata_end(&mut buf, &mut index);
        self.file.write_to_page(current_page, &mut buf).unwrap();
        // println!("after serialization: {:?}", buf[0..100].to_owned());
        return Some(());
    }

    fn deserialize_metadata(mut self) -> Self {
        let mut buf = [0u8; PAGE];
        let mut index = 0;
        let _ = self.file.get_page(0, &mut buf); //check later
        let tmp = deserialize_u8(&mut buf, &mut index);
        assert!(tmp == PageType::MetaData as u8);
        loop {
            let htype = u8to_metadatatype(deserialize_u8(&mut buf, &mut index));
            // println!(" metadataType u8 = {}", htype.clone() as u8);
            match htype {
                MetaDataTypes::End => break,
                MetaDataTypes::Column => {
                    self.columns
                        .push(deserialize_metadata_column(&mut buf, &mut index));
                }
                MetaDataTypes::Index => {
                    self.indexes
                        .push(deserialize_metadata_index(&mut buf, &mut index));
                }
                MetaDataTypes::NextPage => {
                    self.file
                        .get_page(
                            deserialize_metadata_next_page(&mut buf, &mut index) as usize,
                            &mut buf,
                        )
                        .unwrap();
                }
                MetaDataTypes::FreePage => {
                    self.free_page = deserialize_metadata_free_page(&mut buf, &mut index);
                }
            }
        }
        return self;
    }

    pub fn open_table(name: String) -> Option<Self> {
        let dsc = match File::open_file(&name) {
            Ok(tmp) => tmp,
            Err(_) => return None,
        };
        let table = Table::new(dsc);
        Some(table.deserialize_metadata())
    }

    pub fn create_table(name: String, mut columns: Vec<Column>) -> Option<Self> {
        if has_duplicate_column_names(&columns) == true {
            return None;
        }
        let dsc = match File::create_file(&name) {
            Some(tmp) => tmp,
            None => return None,
        };
        columns.insert(
            0,
            Column {
                name: "UID".to_owned(),
                col_type: ColTypes::U64,
            },
        );
        let mut table = Table {
            file: dsc,
            columns: columns,
            indexes: vec![],
            free_page: FreePage { page_index: 1 },
        };

        if table.init_metadata().is_none() {
            File::delete_files(&name);
            return None;
        }
        return Some(table);
    }
}

fn serialize_string(buf: &mut [u8], index: &mut usize, string: &String) {
    let len = string.as_bytes().len();
    serialize_u64(buf, index, len as u64);
    // println!("serialize string");
    // println!("  len: {}", len);
    // println!("  string: {}", string);
    buf[*index..*index + len as usize].copy_from_slice(string.as_bytes()); //string
    *index += len as usize;
}

fn serialize_u8(buf: &mut [u8], index: &mut usize, el: u8) {
    buf[*index] = el;
    *index += 1;
}

fn serialize_u64(buf: &mut [u8], index: &mut usize, el: u64) {
    buf[*index..*index + 8].copy_from_slice(&el.to_le_bytes());
    *index += 8;
}

fn deserialize_string(buf: &mut [u8], index: &mut usize) -> String {
    // println!("deserialize string");
    let len_bytes: [u8; 8] = buf[*index..*index + 8].try_into().unwrap();
    let len = u64::from_le_bytes(len_bytes) as usize;
    // println!(" deserialized string len: {}", len);
    *index += 8;
    let s = std::str::from_utf8(&buf[*index..*index + len])
        .expect("invalid utf-8 in deserialize_string")
        .to_string();
    // println!(" deserialized string: {}", s);
    // println!("  slice: {:?}", &buf[*index - 4..*index + len]);
    *index += len;
    s
}

fn deserialize_u8(buf: &mut [u8], index: &mut usize) -> u8 {
    let ret = buf[*index];
    *index += 1;
    ret
}

fn deserialize_u64(buf: &mut [u8], index: &mut usize) -> u64 {
    let ret_bytes: [u8; 8] = buf[*index..*index + 8].try_into().unwrap();
    let ret = u64::from_le_bytes(ret_bytes) as usize;
    *index += 1;
    ret as u64
}

fn u8to_metadatatype(i: u8) -> MetaDataTypes {
    match i {
        0 => MetaDataTypes::End,
        1 => MetaDataTypes::Column,
        2 => MetaDataTypes::Index,
        3 => MetaDataTypes::NextPage,
        _ => panic!("fucked up metadatatype"),
    }
}

fn u8to_coltype(i: u8) -> ColTypes {
    match i {
        0 => ColTypes::Empty,
        1 => ColTypes::I32,
        2 => ColTypes::U32,
        3 => ColTypes::I64,
        4 => ColTypes::U64,
        5 => ColTypes::String,
        _ => panic!("fucked up col_type"),
    }
}

fn serialize_metadata_end(buf: &mut [u8], index: &mut usize) {
    serialize_u8(buf, index, MetaDataTypes::End as u8);
}

fn serialize_metadata_next_page(buf: &mut [u8], index: &mut usize, new_page: usize) {
    serialize_u8(buf, index, MetaDataTypes::NextPage as u8);
    serialize_u64(buf, index, new_page as u64);
}

fn deserialize_metadata_next_page(buf: &mut [u8], index: &mut usize) -> u64 {
    deserialize_u64(buf, index)
}

fn deserialize_metadata_free_page(buf: &mut [u8], index: &mut usize) -> FreePage {
    FreePage {
        page_index: deserialize_u64(buf, index),
    }
}

fn serialize_metadata_index(buf: &mut [u8], index: &mut usize, i: &Indexing) {
    serialize_u8(buf, index, MetaDataTypes::Index as u8); //type of metadata
    serialize_string(buf, index, &i.name); // string
    serialize_u64(buf, index, i.page_index as u64); //head of index
}

fn serialize_metadata_free_page(buf: &mut [u8], index: &mut usize, i: u64) {
    serialize_u8(buf, index, MetaDataTypes::FreePage as u8); //type of metadata
    serialize_u64(buf, index, i); //head of index
}

fn deserialize_metadata_index(buf: &mut [u8], index: &mut usize) -> Indexing {
    let string = deserialize_string(buf, index);
    let page_index = deserialize_u64(buf, index);
    Indexing {
        name: string,
        page_index: page_index,
    }
}

fn serialize_metadata_column(buf: &mut [u8], index: &mut usize, column: &Column) {
    serialize_u8(buf, index, MetaDataTypes::Column as u8); //type of metadata
    //string
    serialize_string(buf, index, &column.name); // string
    serialize_u8(buf, index, column.col_type.clone() as u8); //head of index
}

fn deserialize_metadata_column(buf: &mut [u8], index: &mut usize) -> Column {
    // let _ = deserialize_u64(buf, index); // deserializing len of index in metadata
    let string = deserialize_string(buf, index);
    let col_type = deserialize_u8(buf, index);
    // println!(
    //     "deserialized column name: |{}|\n col_type u8: {}",
    //     string, col_type,
    // );
    Column {
        name: string,
        col_type: u8to_coltype(col_type),
    }
}

fn has_duplicate_column_names(arr: &Vec<Column>) -> bool {
    let len = arr.len();
    for i in 0..len {
        for j in (i + 1)..len {
            if arr[i].name == arr[j].name {
                return true;
            }
        }
    }
    return false;
}

fn u8_to_page_type(val: u8) -> PageType {
    match val {
        0 => PageType::Empty,
        1 => PageType::DataStructure,
        2 => PageType::Data,
        3 => PageType::MetaData,
        _ => panic!("incorect page_type casting"),
    }
}

fn deserialize_page_type(buf: &mut [u8], index: &mut usize) -> PageType {
    let tmp = deserialize_u8(buf, index);
    u8_to_page_type(tmp)
}

fn serialize_page_type(buf: &mut [u8], index: &mut usize, tp: &PageType) {
    serialize_u8(buf, index, *tp as u8);
}

/*
struct OverflowCell {
    page_index: u64,
    cell_index: u64,
}
*/
