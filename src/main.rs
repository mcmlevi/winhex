use std::{fs::{self}};

use clap::Parser;

/// winhex is a utility tool to visualize binary data 
#[derive(Parser)]
struct Args {
    /// The fileName to use in winhex
    file: std::path::PathBuf,

    /// the hex width
    #[arg(long, default_value_t = 16)]
    width: u16,

    /// The number of rows to display
    #[arg(long, default_value_t = 16)]
    height: u16,

    /// Output the text as UTF-8
    #[arg(long,default_value_t = false)]
    utf8 : bool,
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

fn main() {
    let args = Args::parse();
    
    match fs::read(args.file) {
        Ok(buffer) => {
            format_header(args.width);
            for (i, slice) in buffer.windows(args.width as usize).step_by(args.width as usize).enumerate() {
                format_row(i as u64 * args.width as u64, slice, args.utf8);
                
                //TODO this should prompt for more input possibly
                if i >= args.height as usize {
                    break;
                }
            }
        }
        Err(_) =>{
            println!("File could not be read successfully");
        }
    }
}