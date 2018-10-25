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
use std::env;

//static WEB_SRC_PATH: &'static str = "./web_src/src/";
//static WEB_OUT_PATH: &'static str = "./web_out/";

#[derive(Deserialize)]
struct Block{
    _id: String,
    template_id: String,
    width_percent: u8,
    stylesheet_override: String,
    string_maps: Vec<StringMap>,
    blocks: Vec<Block>,
}

#[derive(Deserialize)]
struct Stylesheet{
    _id: String,
    path: String,
}

#[derive(Deserialize)]
struct Template{
    _id: String,
    _stylesheet_id: String,
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
        fs::create_dir(web_src_path.clone()).unwrap();
        src_paths = fs::read_dir(web_src_path.clone()).unwrap();
        println!("The path {} was not found, created the path!", web_src_path)
    } else {
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
                    let block: Block = read_json_object(&mut file_reader).unwrap();
                    blocks.push(block);
                } else if tag == "TEMPLATE" {
                    let template: Template = read_json_object(&mut file_reader).unwrap();
                    templates.push(template);
                } else if tag == "STYLESHEET" {
                    let stylesheet: Stylesheet = read_json_object(&mut file_reader).unwrap();
                    stylesheets.push(stylesheet);
                }
            } else {
                break;
            }
        }
    }

    // Now we have all the json objects we need (blocks, templates and stylesheets)

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

fn find_deep_block_index(ref_id: String, blocks: &Vec<Block>) -> Result<usize, std::io::Error> {
    let index_res = blocks.iter().position(|r| r._id == ref_id);
    if index_res.is_some() {
        return Ok(index_res.unwrap());
    }

    for block in blocks {
        let find_res = find_deep_block_index(ref_id.clone(), &block.blocks);
        if find_res.is_ok() {
            return Ok(find_res.unwrap());
        }
    }
    return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Could not find block by reference with id: {}", ref_id)));
}

fn write_block(block: Block, templates: &Vec<Template>, stylesheets: &Vec<Stylesheet>, web_src_path: &String, web_out_path: &String) {
    let mut out_file = File::create(web_out_path.to_owned() + &block._id + ".html").unwrap();

    let template_name = block.template_id;

    out_file.write("<!DOCTYPE html>\n<html>\n  <head>\n".as_bytes()).unwrap();

    for t in templates {
        if t._id.eq(&template_name) {

            let stylesheet_id;
            if block.stylesheet_override != "" {
                stylesheet_id = &block.stylesheet_override;
            } else {
                stylesheet_id = &t._stylesheet_id;
            }

            for stylesheet in stylesheets {

                if &stylesheet._id == stylesheet_id {
                    // Found stylesheet
                    let stylesheet_src_path = format!("{}{}", web_src_path, &stylesheet.path);
                    let stylesheet_out_path = format!("{}{}.css", web_out_path, &stylesheet._id);

                    println!("Found stylesheet: {}", stylesheet_src_path);
                    // Copy stylesheet
                    fs::copy(stylesheet_src_path, stylesheet_out_path).unwrap();

                    // Write reference to the stylesheet in head
                    out_file.write(format!("      {}{}{}", "<link rel=\"stylesheet\" href=\"", stylesheet_id ,".css\" />").as_bytes()).unwrap();
                }
            }
        }
    }

    out_file.write("\n  </head>\n  <body>\n".as_bytes()).unwrap();

    //Find corresponding template
    for t in templates {
        if t._id.eq(&template_name) {
            // Found template
            //TODO: include handling if the template is a string

            let template_path = format!("{}{}", web_src_path, t.path);
            print!("Found html template: {}", template_path);

            if template_path != "" {
                let mut template_file = File::open(&template_path).unwrap();
                let mut buf = [0];

                //Write template to file
                loop {
                    let res = template_file.read(&mut buf);
                    if res.is_ok() {
                        let read_bytes = res.unwrap();

                        if read_bytes == 0 {
                            break;
                        }

                        let c = char::from(buf[0]);
                        if c == '#' {
                            let mut control_str = String::new();
                            //read {
                            template_file.read(&mut buf).unwrap();
                            // read first char
                            template_file.read(&mut buf).unwrap();

                            // load first char
                            let mut k = char::from(buf[0]);
                            while k != '}' {
                                control_str.push(k);
                                template_file.read(&mut buf).unwrap();
                                k = char::from(buf[0]);
                            }
                            //read }
                            template_file.read(&mut buf).unwrap();

                            println!("Read control string: {}", control_str);

                            //Find the corresponding string in the block's map
                            for str_map in &(block.string_maps) {
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
        }
    }

    // Sub blocks
    if block.blocks.len() > 0 {

        out_file.write("    <div style=\"width: 100%; display: table;\">\n".as_bytes()).unwrap();
        out_file.write("      <div style=\"display: table-row\">\n".as_bytes()).unwrap();

        //Write all sub blocks as new IFRAMEs
        for subblock in block.blocks
        {
            out_file.write(format!("        <div id=\"{}\" style=\"display: table-cell; width:{}%\">\n", subblock._id, subblock.width_percent).as_bytes()).unwrap();
            out_file.write(format!("          <iframe width=\"100%\" height=\"100%\" frameborder=\"0\" src=\"{}.html\"></iframe>\n", subblock._id).as_bytes()).unwrap();
            out_file.write(format!("        </div>\n").as_bytes()).unwrap();
            write_block(subblock, templates, stylesheets, web_src_path, web_out_path);
        }

        out_file.write(format!("      </div>\n").as_bytes()).unwrap();
        out_file.write(format!("    </div>\n").as_bytes()).unwrap();

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
            return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("File reader reached EOF befmre finding a control tag: {}", reader_res.unwrap_err())));
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
