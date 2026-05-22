//use rand::prelude::*;
use std::env;
//use std::fs::File;
//use std::fs::OpenOptions;
//
//use std::os::unix::fs::FileExt;
//

pub mod btree;
#[cfg(test)]
mod btree_tests;
pub mod table;
use table::ColTypes;
use table::Column;
use table::Table;

fn str_to_col_types(s: &str) -> ColTypes {
    match s {
        "Empty" => ColTypes::Empty,
        "I32" => ColTypes::I32,
        "U32" => ColTypes::U32,
        "I64" => ColTypes::I64,
        "U64" => ColTypes::U64,
        "String" => ColTypes::String,
        _ => panic!("invalid column type"),
    }
}

fn create_table(args: &[String]) -> Option<()> {
    let mut columns: Vec<Column> = vec![];
    for chunk in args[1..].chunks_exact(2) {
        let [name, col_type] = chunk else {
            panic!("missing type of name of column");
        };
        let col_type = str_to_col_types(col_type.as_str());
        let tmp = Column {
            name: name.clone(),
            col_type: col_type,
        };
        columns.push(tmp);
    }
    table::Table::create_table(args[0].clone(), columns);
    None
}

fn main() {
    let mut columns: Vec<Column> = vec![];

    let col = Column {
        name: "string".to_owned(),
        col_type: ColTypes::String,
    };
    columns.push(col);

    let col = Column {
        name: "i32".to_owned(),
        col_type: ColTypes::I32,
    };
    columns.push(col);

    let col = Column {
        name: "u64".to_owned(),
        col_type: ColTypes::U64,
    };
    columns.push(col);

    let mut tab = table::Table::create_table("./database/hehe".to_owned(), columns)
        .expect("Failed to created database");

    let tab =
        table::Table::open_table("./database/hehe".to_owned()).expect("Failed to open database");
    tab.display();
}
