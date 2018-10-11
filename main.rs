use std::fs::File;
use std::io::prelude::*;
use std::io::Error;

static INDEX_FILE_NAME: &'static str = "index.html";
static ERR_DUMP_FILE_NAME: &'static str = "dump";


fn main() {
    
    let mut file = File::create(INDEX_FILE_NAME).unwrap();

    file = write_tag(format!("!DOCTYPE html"), file);


    file = write_tag("html".to_owned(), file);
    file = write_tag("body".to_owned(), file);
    file = write_text("Fuck this, Imma just make porn instead ._.".to_owned(), file);
    file = write_end_tag("body".to_owned(), file);
    write_end_tag("html".to_owned(), file);
    println!("Finished writing to file!");

}

fn write(s: String, mut f: &File) -> Result<(), Error>  {
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
        return f;
    }
    println!("ERROR while writing tag: {}. Continuing writing to dump file...", write_str);
    return File::create(ERR_DUMP_FILE_NAME).unwrap();
}

fn write_end_tag(s: String, f:File) -> File {
    let write_str = format!("</{}>", s);
    let result = write(write_str.clone(), &f);
    if result.is_ok() {
        return f;
    }
    println!("ERROR while writing end tag: {}. Continuing writing to dump file...", write_str);
    return File::create(ERR_DUMP_FILE_NAME).unwrap();
}

fn write_text(s: String, f: File) -> File {
    let result = write(s.clone(), &f);
    if result.is_ok() {
        return f;
    }
    println!("ERROR while writing text: {}. Continuing writing to dump file...", s);
    return File::create(ERR_DUMP_FILE_NAME).unwrap();
}
