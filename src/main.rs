use std::env;

pub mod btree;
#[cfg(test)]
mod btree_tests;
pub mod table;

fn print_usage(bin_name: &str) {
    eprintln!("Usage:");
    eprintln!("  {} create <table_file> <column> [column...]", bin_name);
    eprintln!("  {} metadata <table_file>", bin_name);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let bin_name = args.first().map(String::as_str).unwrap_or("database");

    match args.get(1).map(String::as_str) {
        Some("create") => {
            if args.len() < 4 {
                print_usage(bin_name);
                return;
            }

            let table_name = &args[2];
            let columns = args[3..].to_vec();
            if let Err(err) = table::create_table(table_name, &columns) {
                eprintln!("Failed to create table: {}", err);
            }
        }
        Some("metadata") => {
            if args.len() != 3 {
                print_usage(bin_name);
                return;
            }

            match table::open_table(&args[2]) {
                Ok(table) => {
                    println!("{}", table.formatted_metadata());
                }
                Err(err) => {
                    eprintln!("Failed to read metadata: {}", err);
                }
            }
        }
        _ => {
            print_usage(bin_name);
        }
    }
}
