extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

//use serde_json::{Value, Error};
use std::fs;
use std::str;
use std::fs::File;
use std::io::prelude::*;
use std::io::Read;
use std::io::BufReader;
use std::io::BufRead;

static INDEX_FILE_NAME: &'static str = "index.html";
static ERR_DUMP_FILE_NAME: &'static str = "dump";
static WEB_SRC_PATH: &'static str = "./web_src/";

#[derive(Deserialize)]
struct Block{
    _id: String,
    template_id: String,
    coord_x: u8,
    coord_y: u8,
    width_percent: u8,
    stylesheet_override: String,
    
    string_maps: Vec<StringMap>,
    
    blocks: Vec<Block>,
}

#[derive(Deserialize)]
struct Stylesheet{
    _id: String,
    path: String,
    html: String
}

#[derive(Deserialize)]
struct Template{
    _id: String,
    _stylesheet_id: String,
    path: String,
    html: String,
    string_refs: Vec<String>,
}

#[derive(Deserialize)]
struct StringMap{
    _id: String,
    contents: String,
}

#[derive(Deserialize)]
struct StringCollection{
    strings: Vec<StringMap>,
}

fn main() {
    
    //Read all files in the src path
    let src_paths_result = fs::read_dir(WEB_SRC_PATH);
    let src_paths: std::fs::ReadDir;

    if src_paths_result.is_err() {
        fs::create_dir(WEB_SRC_PATH).unwrap();
        src_paths = fs::read_dir(WEB_SRC_PATH).unwrap();
        println!("The path {} was not found, created the path!", WEB_SRC_PATH)
    } else {
        src_paths = src_paths_result.unwrap();
    }

    for path in src_paths {
        let path_str: String = path.unwrap().path().display().to_string();
        println!("Read webpage source file: {}", path_str);
        let mut src_file = File::open(path_str).unwrap();
        let mut file_reader = BufReader::new(src_file);
        
        loop {
            let res = read_control_tag(&mut file_reader);
            if res.is_ok() {
                let tag = res.unwrap();
                println!("Read tag: {}", tag);
                
                let block: Block = serde_json::from_reader(file_reader).unwrap();
                break;

                // TODO: Write from BufReader to String while counting brackets.
                // Then from String to JSON
                // Example:
                //   let mut s = String::new();
                //   s.push_str("GET / HTTP/1.0\r\n");


            } else {
                break;
            }
        }
        
        

        //let mut file_contents = String::new();

        
        
        
        
        //src_file.read_to_string(&mut file_contents).unwrap();
        //println!("Read file contents: {}", &file_contents);

        // TODO: use to_writer instead: https://docs.serde.rs/serde_json/fn.to_writer.html
        //let b: Block = serde_json::from_str(file_contents.as_str()).unwrap();

        //println!("Read Box: x: {}, y: {}", b.coord_x, b.coord_y);
    }




    let mut file = File::create(INDEX_FILE_NAME).unwrap();

    file = write_tag(format!("!DOCTYPE html"), file);


    file = write_tag("html".to_owned(), file);
    file = write_tag("body".to_owned(), file);
    file = write_text("Fuck this, Imma just make porn instead ._.".to_owned(), file);
    file = write_end_tag("body".to_owned(), file);
    write_end_tag("html".to_owned(), file);
    println!("Finished writing to file!");

}

fn read_control_tag(reader: &mut BufReader<File>) -> Result<String, std::io::Error> {
    let mut buf = [0];
    let reader_res = reader.read(&mut buf);
    let read_bytes: usize;
    if reader_res.is_ok() {
        read_bytes = reader_res.unwrap();
    } else {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("File reader reached EOF befmre finding a control tag: {}", reader_res.unwrap_err())));
    }

    if read_bytes == 0 {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "File reader reached EOR before finding a control tag"));
    }
    
    let c = char::from(buf[0]);
 
    // Control tag
    if c == '#' {
        reader.read(&mut buf).unwrap();
        let mut tag_buf: Vec<u8> = Vec::new();
        reader.read_until(']' as u8, &mut tag_buf).unwrap();
        let l = tag_buf.len();
        tag_buf.truncate(l-1);      // Cut trailing ] on the tag
        let tag = String::from_utf8(tag_buf).unwrap();
        return Ok(tag);
    }
    
    Err(std::io::Error::new(std::io::ErrorKind::Other, "Expected control tag was not found"))

}

fn write(s: String, mut f: &File) -> Result<(), std::io::Error> {
    // {} -> write to_string trait
    // {:?} -> write debug trait, much more common
    // Ok(_) -> "throw away" the value
    // let result = match...   <- must be used
    match f.write_all(&s.into_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => Err(e)
    }

}

fn write_tag(s: String, f: File) -> File {
    let write_str = format!("<{}>", s);
    let result = write(write_str.clone(), &f);
    if result.is_ok() {
        f
    } else {
        println!("ERROR while writing tag: {}. Continuing writing to dump file...", write_str);
        File::create(ERR_DUMP_FILE_NAME).unwrap()
    }
}

fn write_end_tag(s: String, f:File) -> File {
    let write_str = format!("</{}>", s);
    let result = write(write_str.clone(), &f);
    if result.is_ok() {
        f
    } else {
        println!("ERROR while writing end tag: {}. Continuing writing to dump file...", write_str);
        File::create(ERR_DUMP_FILE_NAME).unwrap()
    }
}

fn write_text(s: String, f: File) -> File {
    let result = write(s.clone(), &f);
    if result.is_ok() {
        f
    } else {
        println!("ERROR while writing text: {}. Continuing writing to dump file...", s);
        File::create(ERR_DUMP_FILE_NAME).unwrap()
    }
}
