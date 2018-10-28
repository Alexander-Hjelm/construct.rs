extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use std::fs;
use std::str;
use std::fs::File;
use std::io::prelude::*;
use std::io::Read;
use std::io::BufReader;
use std::io::BufRead;
use std::env;

#[derive(Deserialize)]
struct Block{
    _id: String,
    template_id: String,
    width_percent: u8,
    stylesheet_id: Option<String>,
    string_maps: Option<Vec<StringMap>>,
    blocks: Option<Vec<Vec<Block>>>,
}

#[derive(Deserialize)]
struct Stylesheet{
    _id: String,
    path: String,
}

#[derive(Deserialize)]
struct Template{
    _id: String,
    path: String,
}

#[derive(Deserialize)]
struct StringMap{
    _id: String,
    contents: String,
}

fn main() {

    let mut web_src_path = String::new();
    let mut web_out_path = String::new();

    let mut c = 0;

    if env::args().len() < 3 {
        show_help();
        return;
    }

    for arg in env::args() {
        println!("{}", arg);
        if arg == "-h" || arg == "--help" {
            show_help();
            return;
        }
        if c==1 {
            web_src_path = arg.clone();
        }
        if c==2 {
            web_out_path = arg;
        }

        c+=1;
    }

    // Add trailing / to dirs
    let c = web_src_path.chars().next_back().unwrap();
    if c != '/' {
        web_src_path = format!("{}{}", web_src_path, '/');
    }

    let c = web_out_path.chars().next_back().unwrap();
    if c != '/' {
        web_out_path = format!("{}{}", web_out_path, '/');
    }

    //Read all files in the src path
    let src_paths_result = fs::read_dir(web_src_path.clone());
    let src_paths: std::fs::ReadDir;

    if src_paths_result.is_err() {
        let create_dir_res = fs::create_dir(web_src_path.clone());
        if create_dir_res.is_ok() {
            let read_dir_res = fs::read_dir(web_src_path.clone());
            if read_dir_res.is_ok() {
                src_paths = read_dir_res.unwrap();
            } else {
                println!("ERROR! Could not read from the directory: {}. Make sure you have access rights", web_src_path);
                return;
            }
        } else {
            println!("ERROR! The path {} could not be created. Make sure you have access rights or create the directory manually", web_src_path);
            return;
        }
        println!("The path {} was not found, created the path!", web_src_path)
    } else {
        // Read files Ok!
        src_paths = src_paths_result.unwrap();
    }

    let mut blocks = Vec::new();
    let mut templates = Vec::new();
    let mut stylesheets = Vec::new();

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

                if tag == "BLOCK" {
                    let block_res = read_json_object(&mut file_reader);
                    if block_res.is_ok() {
                        let block: Block = block_res.unwrap();
                        blocks.push(block);
                    } else {
                        println!("Error while reading block: {}", block_res.err().unwrap());
                    }
                } else if tag == "TEMPLATE" {
                    let template_res = read_json_object(&mut file_reader);
                    if template_res.is_ok() {
                        let template: Template = template_res.unwrap();
                        templates.push(template);
                    } else {
                        println!("Error while reading template: {}", template_res.err().unwrap());
                    }
                } else if tag == "STYLESHEET" {
                    let stylesheet_res = read_json_object(&mut file_reader);
                    if stylesheet_res.is_ok() {
                        let stylesheet: Stylesheet = stylesheet_res.unwrap();
                        stylesheets.push(stylesheet);
                    } else {
                        println!("Error while reading stylesheet: {}", stylesheet_res.err().unwrap());
                    }
                }
            } else {
                println!("{}", res.unwrap_err());
                break;
            }
        }
    }

    // Now we have all the json objects we need (blocks, templates and stylesheets)

    //Duplicate id check on templates
    let mut i = 0;
    let mut j = 0;
    for t1 in &templates {
        for t2 in &templates {
            if i!=j && t1._id.eq(&t2._id) {
                println!("There are two templates with the same _id property. (_id = {})", t2._id);
                return;
            }
            j+=1;
        }
        j=0;
        i+=1;
    }

    //Duplicate id check on stylesheets
    i = 0;
    j = 0;
    for ss1 in &stylesheets {
        for ss2 in &stylesheets {
            if i != j && ss1._id.eq(&ss2._id) {
                println!("There are two stylesheets with the same _id property. (_id = {})", ss2._id);
                return;
            }
            j+=1;
        }
        j=0;
        i+=1;
    }

    // Create dir
    let res = fs::create_dir(web_out_path.clone());
    if res.is_err() {println!("Out path already exists. Continuing happily...")}

    for block in blocks {
        //Write template to file
        write_block(block, &templates, &stylesheets, &web_src_path, &web_out_path);
    }
    println!("Finished writing to file!");

}

fn show_help() {
    println!("Usage:\n  construct <src-path> <out-path>\n\nWhere:\n  <src-path> is the directory
    that contains the JSON website definitions, stylesheets and html template files.\n  <out-path>
    is the directory that construct shall write your website html files to.")
}

fn write_block(block: Block, templates: &Vec<Template>, stylesheets: &Vec<Stylesheet>, web_src_path: &String, web_out_path: &String) {
    let mut out_file = File::create(web_out_path.to_owned() + &block._id + ".html").unwrap();

    let template_name = block.template_id;

    out_file.write("<!DOCTYPE html>\n<html>\n  <head>\n".as_bytes()).unwrap();

    if block.stylesheet_id.is_some() {
        let stylesheet_id = block.stylesheet_id.unwrap();

        for stylesheet in stylesheets {

            if stylesheet._id == stylesheet_id {
                // Found stylesheet
                let stylesheet_src_path = format!("{}{}", web_src_path, &stylesheet.path);
                let stylesheet_out_path = format!("{}{}.css", web_out_path, &stylesheet._id);

                println!("Found stylesheet: {}", stylesheet_src_path);
                // Copy stylesheet
                let copy_res = fs::copy(&stylesheet_src_path, &stylesheet_out_path);
                if copy_res.is_err() {
                    println!("Error while copying stylesheet file: {} to path: {}.
                    Make sure that the file and the target directory exist.", stylesheet_src_path,
                    stylesheet_out_path);
                    return;
                }

                // Write reference to the stylesheet in head
                out_file.write(format!("      {}{}{}", "<link rel=\"stylesheet\" href=\"", stylesheet_id ,".css\" />").as_bytes()).unwrap();
            }
        }
    }

    out_file.write("\n  </head>\n  <body>\n".as_bytes()).unwrap();

    let str_maps = block.string_maps;;

    let mut template_file = Err(std::io::Error::new(std::io::ErrorKind::Other, "No template file found with that id."));;
    let mut buf = [0];

    //Find corresponding template
    for t in templates {
        if t._id.eq(&template_name) {
            // Found template
            //TODO: include handling if the template is a string

            let template_path = format!("{}{}", web_src_path, t.path);
            println!("Found html template: {}", template_path);

            if template_path != "" {
                template_file = File::open(&template_path);
                break;
            }
        }
    }

    let sm_some = str_maps.unwrap_or(vec!());

    if template_file.is_ok() {
        let mut source_file = template_file.unwrap();
        //Write template to file
        loop {

            let res = source_file.read(&mut buf);
            if res.is_ok() {
                let read_bytes = res.unwrap();

                if read_bytes == 0 {
                    break;
                }

                let c = char::from(buf[0]);
                if c == '#' {
                    let mut control_str = String::new();
                    //read {
                    source_file.read(&mut buf).unwrap();
                    // read first char
                    source_file.read(&mut buf).unwrap();

                    // load first char
                    let mut k = char::from(buf[0]);
                    while k != '}' {
                        control_str.push(k);
                        source_file.read(&mut buf).unwrap();
                        k = char::from(buf[0]);
                    }
                    //read }
                    source_file.read(&mut buf).unwrap();

                    println!("Read control string: {}", control_str);

                    //Find the corresponding string in the block's map
                    for str_map in &sm_some {
                        if str_map._id.eq(&control_str) {
                            let smac = str_map.contents.clone();
                            let bytes = smac.into_bytes();
                            out_file.write_all(&bytes).unwrap();
                        }
                    }
                }
                out_file.write(&buf).unwrap();
            } else {
                break;
            }
        }
    }
    else {
        println!("ERROR while reading template file. Error message was: {}", template_file.unwrap_err());
        return;
    }

    // Sub blocks
    if block.blocks.is_some() {
        let subblocks2D = block.blocks.unwrap();
        if subblocks2D.len() > 0 {
            println!("block: {}, has {} sub block arrays", block._id, subblocks2D.len());
            //Write all sub blocks as new IFRAMEs
            for subblocks in subblocks2D {
                println!("read one sub block array with lenght: {}", subblocks.len());

                if subblocks.len() > 0 {
                    out_file.write("    <div style=\"width: 100%; display: table;\">\n".as_bytes()).unwrap();
                    out_file.write("      <div style=\"display: table-row\">\n".as_bytes()).unwrap();

                    println!("writing table row");

                    for subblock in subblocks {
                        println!("writing subblock: {}", subblock._id);

                        out_file.write(format!("        <div id=\"{}\" style=\"display: table-cell; line-height: 0px; width:{}%\">\n", subblock._id, subblock.width_percent).as_bytes()).unwrap();
                        out_file.write(format!("          <iframe width=\"100%\" height=\"100%\" frameborder=\"0\" src=\"{}.html\"></iframe>\n", subblock._id).as_bytes()).unwrap();
                        out_file.write(format!("        </div>\n").as_bytes()).unwrap();
                        write_block(subblock, templates, stylesheets, web_src_path, web_out_path);
                    }
                    out_file.write(format!("      </div>\n").as_bytes()).unwrap();
                    out_file.write(format!("    </div>\n").as_bytes()).unwrap();
                }
            }
        }
    }
    out_file.write("  </body>\n</html>\n".as_bytes()).unwrap();
}

fn read_json_object<T>(reader: &mut BufReader<File>) -> Result<T, std::io::Error> where T: serde::de::DeserializeOwned {
    let mut bracket_count = 0;
    let mut buf = [0];

    let mut out_str = String::new();

    //Read until first bracket
    loop {
        let reader_res = reader.read(&mut buf);
        if reader_res.is_ok() {
            reader_res.unwrap();
        } else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "File reader reached EOF before finding a json object"));
        }
        let c = char::from(buf[0]);
        if c == '{' {
            bracket_count = 1;
            out_str.push(c);
            break;
        }
    }

    loop {
        let reader_res = reader.read(&mut buf);
        if reader_res.is_ok() {
            reader_res.unwrap();
        } else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "File reader reached EOF while reading a json object"));
        }
        let c = char::from(buf[0]);
        if c == '{' {
            bracket_count = bracket_count + 1;
        } else if c == '}' {
            bracket_count = bracket_count - 1;
        }
        out_str.push(c);

        if bracket_count == 0 {
            break;
        }

    }

    let immutable_str = &out_str; //This is a &String type
    let out_str_native: &str = &immutable_str; //This is an &str type

    println!("{}", out_str_native);

    // String to JSON
    let res = serde_json::from_str(out_str_native);
    match res {
        Ok(_) => Ok(res.unwrap()),
        Err(e) => Err(e.into())     // Convert serde_json::Error to io::Error
    }
}

fn read_control_tag(reader: &mut BufReader<File>) -> Result<String, std::io::Error> {
    let mut buf = [0];
    loop {
        let reader_res = reader.read(&mut buf);
        let read_bytes: usize;
        if reader_res.is_ok() {
            read_bytes = reader_res.unwrap();
        } else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("File reader reached EOF before finding a control tag: {}", reader_res.unwrap_err())));
        }

        if read_bytes == 0 {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "File reader reached EOR before finding a control tag"));
        }

        let c = char::from(buf[0]);
        if c == '#' {
            break;
        }
    }

    // Control tag
    reader.read(&mut buf).unwrap();
    let mut tag_buf: Vec<u8> = Vec::new();
    reader.read_until(']' as u8, &mut tag_buf).unwrap();
    let l = tag_buf.len();
    tag_buf.truncate(l-1);      // Cut trailing ] on the tag
    let tag = String::from_utf8(tag_buf).unwrap();
    return Ok(tag);
}
