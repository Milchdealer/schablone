use std::fs::{self, DirEntry};
use std::path::Path;

use tera::{Context, Tera};

use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum SchabloneError {
    // #[snafu(display("Text: {}", message))]
    // ExecutionError {
    //     message: String,
    // },
    #[snafu(display("Failed to template"))]
    TemplateError,
}


pub fn new_schablone(name: &str) {
    if let Err(e) = fs::create_dir(name) {
        println!("Failed to create directory {}: {}!", name, e);
        ::std::process::exit(1);
    }

    // Todo: Put default README/files into folder
}

// Parses a string containing KEY=VALUE pairs, separated by comma.
fn parse_parameters(parameters: &str) -> Context {
    println!("Parsing tera context from parameters: '{}'", parameters);
    let mut context = Context::new();

    for pair in parameters.split(",") {
        let mut kv = pair.split("=");
        let key = match kv.next() {
            Some(k) => k,
            None => {
                println!("Invalid key-value pair: {}", pair);
                continue;
            }
        };
        let value = match kv.next() {
            Some(v) => v,
            None => {
                println!("Invalid key-value pair: {}", pair);
                continue;
            }
        };

        context.insert(key.to_owned(), &value.to_owned());
    }

    context
}

fn template_pathname(dir: &Path, base_path: &Path, tera: &mut Tera, context: &Context) -> Result<String, SchabloneError> {
    // Add the directory/file name for templating
    let full_path = base_path.join(dir);
    let full_path = match full_path.to_str() {
        Some(s) => s,
        None => {
            println!("Failed to convert path to string");
            return Err(SchabloneError::TemplateError);
        }
    };
    let name = match dir.to_str() {
        Some(s) => s,
        None => {
            println!("Failed to convert pathname to string");
            return Err(SchabloneError::TemplateError);
        }
    };
    let mut tera_key: String = "schablone://".to_owned();
    tera_key.push_str(full_path);
    println!("tera_key: {}", tera_key);
    println!("name: {}", name);
    if let Err(e) = tera.add_raw_template(&tera_key, name) {
        println!("Failed to add raw template for filename: {}", e);
        return Err(SchabloneError::TemplateError);
    }

    let result = match tera.render(&tera_key, &context) {
        Ok(result) => result,
        Err(e) => {
            println!("Failed to render tera template: {}", e);
            return Err(SchabloneError::TemplateError);
        }
    };
    println!("Tera render result: {}", result);
    Ok(result)
}

fn process_directory(dir: &Path, base_path: &Path, cb: &ProcessingFunction, tera: &mut Tera, context: &Context) {
    let templated_pathname = template_pathname(dir, base_path, tera, context);
    if dir.is_dir() {  // dir does not have the full name!!
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                let child_path = base_path.join(dir);
                process_directory(&path, &child_path, cb, tera, context);
            } else {
                cb(&entry, &base_path, tera, context);
            }
        }
    }
}

type ProcessingFunction = dyn Fn(&DirEntry, &Path, &mut Tera, &Context);
fn process_file(entry: &DirEntry, base_path: &Path, tera: &mut Tera, context: &Context) {
    // Join path with target dir
    if let Err(e) = fs::create_dir(entry.path()) {
        println!("Failed to create directory: {}!", e);
    }
}

pub fn build_schablone(name: &str, target: &str, parameters: &str) {
    println!("Creating target directory '{}'", target);
    if let Err(e) = fs::create_dir(target) {
        println!("Failed to create directory '{}': {}!", target, e);
        ::std::process::exit(1);
    }

    let context = parse_parameters(parameters);
    let mut path: String = name.to_owned();
    path.push_str(&"/**/*".to_owned());
    println!("Parsing schablone from {}", path);
    let mut tera = match Tera::new(&path) {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        },
    };

    let source_path = Path::new(name);
    let target_path = Path::new(target);
    process_directory(source_path, target_path, &process_file, &mut tera, &context);
}
