extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

//use serde_json::{Value, Error};
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::Read;

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
        let mut file_contents = String::new();
        src_file.read_to_string(&mut file_contents).unwrap();
        println!("Read file contents: {}", &file_contents);

        // TODO: use to_writer instead: https://docs.serde.rs/serde_json/fn.to_writer.html
        let b: Block = serde_json::from_str(file_contents.as_str()).unwrap();

        println!("Read Box: x: {}, y: {}", b.coord_x, b.coord_y);
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

fn write(s: String, mut f: &File) -> Result<(), std::io::Error>  {
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
