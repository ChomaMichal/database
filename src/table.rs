use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;

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
*]
* how to implement btree
* use the first page for  metadata
* figure out how to handle collum insertions without copying the whole table or how it might be OK
*/

//might change later

#[derive(Debug, Clone)]
pub struct Table {
    name: String,      //to find the file in question
    rows: Vec<String>, // names of the columns index is important because cells use this indexing
    //this will contain the b tree for searching elements
    free_page: u64,
}

#[derive(Debug, Clone)]
struct Data {
    pointer: u64, //contains the pointer to the data in context of this page
    len: u64,     //length of data might delete later
}

#[derive(Debug, Clone)]
struct Cell {
    index: u64,      //contains the index of the row being quueried
    rows: Vec<Data>, //contains pinter to begining of the data in the same indexing as in the vec
    //in table
    len: u64,
    //cont:  OverflowCell, //figure out how tf to handle overflow cell
}

/*
struct OverflowCell {
    page_index: u64,
    cell_index: u64,
}
*/

pub enum Types {
    Empty,
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    String(String),
}
pub type Row = Vec<Types>;

fn get_table(name: &str) -> Result<File, std::io::Error> {
    OpenOptions::new().write(true).read(true).open(name)
}

pub fn add_column(name: &String, data: Row) -> Result<(), ()> {
    let mut file = match get_table(name) {
        Ok(val) => val,
        Err(e) => return Err(()),
    };
    return Err(());
}

pub fn create_table(name: &str, columns: &Vec<String>) -> Result<(), String> {
    if Path::new(name).exists() == true {
        return Err("Table already exists".to_string());
    }
    let mut file = match OpenOptions::new().write(true).create_new(true).open(name) {
        Ok(val) => val,
        Err(err) => {
            return Err(err.to_string());
        }
    };

    let mut buffer: [u8; PAGE] = [0; PAGE];

    let mut offset = 0usize;
    for s in columns {
        let b = s.as_bytes();
        if offset + b.len() + 1 > PAGE {
            return Err("Table metadata does not fit into the first page".to_string());
        }
        buffer[offset..offset + b.len()].copy_from_slice(b);
        offset += b.len();
        buffer[offset] = b'\n';
        offset += 1;
    }

    match file.write_all(&buffer) {
        Ok(()) => {
            println!("Table created succsessfuly");
        }
        Err(val) => {
            return Err(format!("Failed to write to a file: {}", val));
        }
    }
    return Ok(());
}

pub fn open_table(name: &str) -> Result<Table, String> {
    let mut file = get_table(name).map_err(|err| err.to_string())?;
    let mut buffer: [u8; PAGE] = [0; PAGE];
    file.read_exact(&mut buffer).map_err(|err| err.to_string())?;

    let metadata_end = buffer.iter().position(|&byte| byte == 0).unwrap_or(PAGE);
    let metadata = &buffer[..metadata_end];

    let mut rows = Vec::new();
    for line in metadata.split(|&byte| byte == b'\n') {
        if line.is_empty() {
            continue;
        }
        let value = std::str::from_utf8(line)
            .map_err(|err| err.to_string())?
            .to_string();
        rows.push(value);
    }

    Ok(Table {
        name: name.to_string(),
        rows: rows,
        free_page: 1,
    })
}

impl Table {
    pub fn formatted_metadata(&self) -> String {
        let mut output = String::new();
        output.push_str("Table metadata\n");
        output.push_str("==============\n");
        output.push_str(&format!("name: {}\n", self.name));
        output.push_str(&format!("free_page: {}\n", self.free_page));
        output.push_str("columns:\n");

        if self.rows.is_empty() {
            output.push_str("  (none)\n");
        } else {
            for (idx, column) in self.rows.iter().enumerate() {
                output.push_str(&format!("  {}. {}\n", idx + 1, column));
            }
        }

        output
    }
}
