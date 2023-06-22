use std::fmt;
use std::{fs::{self}, io::{stdin}};

use clap::Parser;
use clap_num::maybe_hex;

use colored::{Colorize};

mod text_highlighter;
use text_highlighter::*;

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

    /// A string to find
    #[arg(long, default_value_t = String::new(), conflicts_with = "find_hex_values")]
    find_text: String,

    // Find a series of hex numbers
    #[arg(long, default_value_t = String::new(), conflicts_with = "find_text")]
    find_hex_values: String,
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
    
        println!("{row}");
    }
}

fn format_row_with_find_results(offset : u64, slice : &[u8], highlight_indexes : &[HighlightIndex], highlighter : &Box<dyn TextHighligher>) {
    let mut row = String::new();
    row += &format!("{:0>8x}   ", offset);

    struct OutputMask
    {
        character: char,
        color: bool,
    }

    let mut output_masks: Vec<OutputMask> =  Vec::new();
    let highlight_location = highlighter.get_highlight_location();


    for (i, elem) in slice.iter().enumerate() {
        let offset_in_slice = i + offset as usize;
        match highlight_location {
            HighLightLocation::Data | HighLightLocation::DataAndText => {
                if highlighter.index_matches_highlight_index(offset_in_slice, highlight_indexes) {
                    row += &(format!("{:02x} ", elem).on_blue());
                } else {
                    row += &format!("{:02x} ", elem);
                }
            },
            HighLightLocation::Text => {
                row += &format!("{:02x} ", elem);
            },
        }

        let mut output_char = *elem;
        // Replace non printable characters to a . and all characters that are not normal ASCII.
        if elem < &(' ' as u8) || elem > &('~' as u8) {
            output_char = '.' as u8;
        }

        match highlight_location {
            HighLightLocation::Text => {
                if highlighter.index_matches_highlight_index(offset_in_slice, highlight_indexes) {
                   output_masks.push(OutputMask { character: output_char as char, color: true });
                   //output_slice += &format!("{}", (output_char as char).to_string().on_blue());
                } else {
                    output_masks.push(OutputMask { character: output_char as char, color: false });
                }
            },
            _ => {
                output_masks.push(OutputMask { character: output_char as char, color: false });
            }
        }
    }

    print!("{row}");
    
    for mask in output_masks {
        if mask.color {
            print!("{}", mask.character.to_string().on_blue());
        } else {
            print!("{}", mask.character.to_string());
        }
    }

    print!("\n");
}

fn print_find_results(args: &Args, buffer: Vec<u8>, highlighter : Box<dyn TextHighligher>) {
    format_header(args.width);

    let offsets = highlighter.match_pattern(&buffer);
   
    for (i, highlight_index) in offsets.iter().enumerate() {
        // Skip elements smaller then the offset.
        if highlight_index.offset + highlight_index.length < args.offset && args.offset != 0 {
            continue;
        }

        for y in 0..args.height {
            let mut max_width = args.width as usize;
            if highlight_index.offset + y as usize * args.width as usize + args.width as usize >= buffer.len() {
                max_width = buffer.len() - highlight_index.offset as usize + y as usize * args.width as usize;
            }

            let slice_start = highlight_index.offset + y as usize * args.width as usize;
            format_row_with_find_results(slice_start as u64, &buffer[slice_start..slice_start + max_width], &offsets[i..], &highlighter);
        }
    }
}

fn print_document_normal(args: &Args, buffer: Vec<u8>) {
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

fn print_document(args: &Args, buffer: Vec<u8>, highlighter : Option<Box<dyn TextHighligher>>) {
    
    match highlighter {
        Some(h) => print_find_results(args, buffer, h),
        None => print_document_normal(args, buffer),
    }   
}

fn validate_input(args: &Args, buffer: &Vec<u8>) -> Result<(), ValidationError> {
    if args.offset as usize >= buffer.len() {
        return Err(ValidationError { error: format!("The file offset: {}, is to large, max size is: {}", args.offset, buffer.len()) });
    }

    Ok(())
}

fn get_highlighter(args: &Args) -> Option<Box<dyn TextHighligher>> {

    if !args.find_text.is_empty() {
        return Some(Box::new(FindOnText { text_to_find: args.find_text.clone()} ))
    }

    if !args.find_hex_values.is_empty() {
        return Some(Box::new(FindOnHexValues { hex_values: Vec::new() }))
    }
    None
}

fn main() {
    let args = Args::parse();

    match fs::read(&args.file) {
        Ok(buffer) => {
            match validate_input(&args, &buffer) {
                Ok(_) => {
                    let highlighter = get_highlighter(&args);
                    print_document(&args, buffer, highlighter);
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