use std::fmt;
use std::{fs::{self}, io::{stdin}};

use clap::Parser;
use clap_num::maybe_hex;

/// winhex is a utility tool to visualize binary data 
#[derive(Parser)]
struct Args {
    /// The fileName to use in winhex
    file: std::path::PathBuf,

    /// the hex width
    #[arg(long, default_value_t = 16)]
    width: u16,

    /// The number of rows to display
    #[arg(long, default_value_t = 16, conflicts_with = "no_limit")]
    height: u16,

    /// Dump the entire file to disk
    #[arg(long = "no-limit", default_value_t = false, conflicts_with  = "height")]
    no_limit: bool,

    /// Output the text as UTF-8
    #[arg(long,default_value_t = false)]
    utf8: bool,

    /// file offset specifier in bytes.
    #[arg(long, default_value_t = 0, value_parser = maybe_hex::<usize>)]
    offset: usize,
}

#[derive(Debug)]
struct ValidationError{error: String}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.error)
    }
}

fn format_header(width : u16) {
    let mut header = String::new();
    header += "Offset (d) ";

    for i in 0..width  {
        header += &format!{"{:>2}",i.to_string()};
        header += " ";
    }

    header += "Decoded text";

    println!("{header}\n");
}

fn format_row(offset : u64, slice : &[u8], is_utf8 : bool) {
    let mut row = String::new();
    row += &format!("{:0>8x}   ", offset);
    
    let mut mutable_slice = slice.to_vec();
    if is_utf8{
        
        for elem in mutable_slice.iter_mut() {
            row += &format!("{:02x} ", elem);
        }
    }
    else {
        for elem in mutable_slice.iter_mut() {
            row += &format!("{:02x} ", elem);
    
            // Replace non printable characters to a . and all characters that are not normal ASCII.
            if *elem < ' ' as u8 || *elem > '~' as u8 {
                *elem = '.' as u8;
            }
        }
    
        row += &String::from_utf8_lossy(&mutable_slice);
    
        println!("{row}\n");
    }
}

fn print_document(args: &Args, buffer: Vec<u8>) {
    format_header(args.width);
    for (i, slice) in buffer[args.offset as usize..].windows(args.width as usize).step_by(args.width as usize).enumerate() {
    
        let index_with_offset = (i * args.width as usize) + args.offset;

        format_row(index_with_offset as u64, slice, args.utf8);
    

        if !args.no_limit {
            if i != 0 && (i % args.height as usize == 0) {
                println!("Show the next set of bytes [y/n]?");
    
                let mut repeat : String = String::new();
                stdin().read_line(&mut repeat).unwrap();
                repeat = repeat.to_lowercase();
    
                match repeat.trim() {
                    "yes" | "y" => {}
                    _ => break,
                };
            }
        }
    }
}

fn validate_input(args: &Args, buffer: &Vec<u8>) -> Result<(), ValidationError> {
    if args.offset as usize >= buffer.len() {
        return Err(ValidationError { error: format!("The file offset: {}, is to large, max size is: {}", args.offset, buffer.len()) });
    }

    Ok(())
}

fn main() {
    let args = Args::parse();
    
    match fs::read(&args.file) {
        Ok(buffer) => {
            match validate_input(&args, &buffer) {
                Ok(_) => {
                    print_document(&args, buffer);
                },
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }
        Err(_) =>{
            println!("File \"{}\" could not be read successfully", args.file.display());
        }
    }
}