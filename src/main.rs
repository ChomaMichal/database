use std::env;
use std::fs::File;
use std::fs::OpenOptions;
use std::os::unix::fs::FileExt;
pub mod table;
use table::PAGE;

fn write_page(file: &File, index: u64, arg: &[u8]) {
    println!("In write page");
    match file.write_all_at(arg, index * PAGE as u64) {
        Err(val) => {
            println!("failed to read file:{}", val);
        }
        Ok(()) => {
            println!("Successfully write to file");
        }
    }
    println!("Leaving write page");
}

fn read_page(file: &File, index: u64) {
    let mut buf: [u8; PAGE] = [0; PAGE];
    match file.read_exact_at(&mut buf, index * PAGE as u64) {
        Err(val) => {
            println!("failed to read file:{}", val);
        }
        Ok(()) => {
            print!(
                "printing page content: {}",
                std::str::from_utf8(&buf).unwrap()
            );
        }
    }
}

fn main() {
    let mut args = env::args().skip(1);
    let arg1 = args.next();
    match arg1.as_deref() {
        None => {
            println!("No arguments");
        }
        Some("new") => {
            let filename = match args.next() {
                Some(name) => name,
                None => {
                    println!("give me file name");
                    return;
                }
            };
            let path = format!("{}{}", "database/", filename);
            let mut colums: Vec<String> = Vec::new();
            loop {
                let column = match args.next() {
                    Some(val) => val,
                    None => break,
                };
                colums.push(column);
            }
            match table::create_table(&path, &colums) {
                Err(e) => println!("Failed to create table: {}", e),
                Ok(()) => println!("Table created Successfully"),
            }
        }
        Some("read") => {
            let filename = match args.next() {
                Some(s) => s,
                None => {
                    println!("give filename");
                    panic!();
                }
            };
            let index: u64 = match args.next() {
                Some(s) => s.parse().expect("give index"),
                None => {
                    println!("give index");
                    panic!();
                }
            };
            let file = File::open(format!("database/{}", filename)).unwrap();

            read_page(&file, index);
        }
        Some("write") => {
            let filename = match args.next() {
                Some(s) => s,
                None => {
                    println!("give filename");
                    panic!();
                }
            };
            let index: u64 = match args.next() {
                Some(s) => s.parse().expect("give index"),
                None => {
                    println!("give index");
                    panic!();
                }
            };
            let file = OpenOptions::new()
                .write(true)
                .open(format!("database/{}", filename))
                .unwrap();
            println!("here");

            write_page(&file, index, &args.next().unwrap().as_bytes());
        }
        _ => {
            println!("expected arguments");
        }
    }
}
