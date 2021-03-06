use std::fs::{self, DirEntry, File};
use std::io::prelude::*;
use std::path::Path;

use serde_json;

use tera::{Context, Tera, Value};

use snafu::Snafu;

/// Errors thrown by this module.
#[derive(Debug, Snafu)]
pub enum SchabloneError {
    #[snafu(display("Failed to template"))]
    TemplateError,
    #[snafu(display("Failed to process file or directory: {}", name))]
    ProcessingError { name: String },
    #[snafu(display("Failed to create file or directory"))]
    FileError,
}

/// Parse a JSON file containing parameters.
///
/// Parses a JSON file containing parameters and returns a [`Context`].
///
/// [`Context`]: tera::Context
fn parse_parameters_file(parameters_file: &str) -> Context {
    info!("Parsing tera context from file: '{}'", parameters_file);
    let content = fs::read_to_string(parameters_file).unwrap_or("{}".to_owned());
    let content: Value = match serde_json::from_str(&content) {
        Ok(content) => content,
        Err(e) => {
            error!("Failed to parse JSON from string: {}", e);
            return Context::new();
        }
    };

    debug!("Parsed JSON: {:?}", content);
    Context::from_value(content).unwrap_or_default()
}

/// Parse parameters given as a `KEY1=VALUE1,KEY2=VALUE2,...` `&str`
///
/// Parses a string containing KEY=VALUE pairs, separated by comma.
/// It returns a [`Context`].
///
/// [`Context`]: tera::Context
fn parse_parameters(parameters: &str) -> Context {
    info!("Parsing tera context from parameters: '{}'", parameters);
    let mut context = Context::new();

    for pair in parameters.split(",") {
        let mut kv = pair.split("=");
        let key = match kv.next() {
            Some(k) => k,
            None => {
                warn!("Invalid key-value pair: {}", pair);
                continue;
            }
        };
        let value = match kv.next() {
            Some(v) => v,
            None => {
                warn!("Invalid key-value pair: {}", pair);
                continue;
            }
        };

        debug!("Inserting key=value pair: {}={}", key, value);
        context.insert(key.to_owned(), &value.to_owned());
    }

    context
}

/// Templates a path using [`Tera`] and the [`Context`].
///
/// Given a [`Tera`] instance and a [`Context`], template the `path` passed.
/// This does a one-off render with tera using the [`render_str`] method.
///
/// [`Tera`]: tera::Tera
/// [`Context`]: tera::Context
/// [`render_str`]: tera::Tera::render_str
fn template_pathname(
    path: &Path,
    tera: &mut Tera,
    context: &Context,
) -> Result<String, SchabloneError> {
    // Add the directory/file name for templating
    let name = match path.to_str() {
        Some(s) => s,
        None => {
            error!("Failed to convert pathname to string");
            return Err(SchabloneError::TemplateError);
        }
    };

    let result = match tera.render_str(&name, &context) {
        Ok(result) => result,
        Err(e) => {
            error!("Failed to render tera template: {}", e);
            return Err(SchabloneError::TemplateError);
        }
    };
    debug!("Tera render result: {}", result);
    Ok(result)
}

/// Recursively process a directory.
///
/// Processing a directory means it creates the directory in the target destination, templating the
/// name using [`template_pathname`] and calling the passed [`ProcessingFunction`] on every file.
///
/// [`template_pathname`]: self::template_pathname
/// [`ProcessingFunction`]: self::ProcessingFunction
fn process_directory(
    dir: &Path,
    source_base: &Path,
    target_base: &Path,
    cb: &ProcessingFunction,
    tera: &mut Tera,
    context: &Context,
    dry_run: bool,
) -> Result<(), SchabloneError> {
    let templated_path = match template_pathname(dir, tera, context) {
        Ok(result) => result,
        Err(e) => {
            let name = dir.file_name().unwrap().to_str().unwrap().to_owned();
            error!("Failed to process {}: {}", name, e);
            return Err(SchabloneError::ProcessingError { name });
        }
    };
    let templated_path = Path::new(&templated_path);
    if dir.is_dir() {
        let target_dir = target_base.join(templated_path);
        if !dry_run {
            if let Err(e) = fs::create_dir(target_dir) {
                error!(
                    "Failed to create directory '{}': {}!",
                    templated_path.to_str().unwrap(),
                    e
                );
                return Err(SchabloneError::ProcessingError {
                    name: templated_path.to_str().unwrap().to_owned(),
                });
            }
        }
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                if let Err(e) =
                    process_directory(&path, source_base, target_base, cb, tera, context, dry_run)
                {
                    error!(
                        "Failed to process entry '{}': {}",
                        path.to_str().unwrap(),
                        e
                    );
                }
            } else {
                if let Err(e) = cb(&entry, &source_base, &target_base, tera, context, dry_run) {
                    error!("Failed to process file: {}", e);
                }
            }
        }
    }
    Ok(())
}

/// Processing function called for every file processed.
///
/// Brevity typedef which should always match the definition of [`process_file`], or any other callback
/// that should be called upon a file.
///
/// [`process_file`]: self::process_file
type ProcessingFunction =
    dyn Fn(&DirEntry, &Path, &Path, &mut Tera, &Context, bool) -> Result<(), SchabloneError>;
/// Process one file.
///
/// Standard callback which processes one file from the schablone.
/// Templates the file's name (with [`template_pathname`]) and content (with [`render`]) and copies it over to the destination.
///
/// [`template_pathname`]: self::template_pathname
/// [`render`]: tera::Tera::render
fn process_file(
    entry: &DirEntry,
    source_base: &Path,
    target_base: &Path,
    tera: &mut Tera,
    context: &Context,
    dry_run: bool,
) -> Result<(), SchabloneError> {
    let path = entry.path();
    // Tera strips the root in the template's key, so we need to strip it too
    let path_name = path.strip_prefix(source_base).unwrap();
    let path_name = path_name.to_str().unwrap();
    debug!("Path: {}", path_name);
    let templated_pathname = match template_pathname(&path, tera, context) {
        Ok(result) => result,
        Err(e) => {
            let name = entry.file_name().to_str().unwrap().to_owned();
            error!("Failed to process {}: {}", name, e);
            return Err(SchabloneError::ProcessingError { name });
        }
    };
    let templated_path = Path::new(&templated_pathname);
    let content = match tera.render(path_name, &context) {
        Ok(content) => content,
        Err(e) => {
            error!("Failed to template '{}': {}", path_name, e);
            return Err(SchabloneError::TemplateError);
        }
    };
    let templated_path = target_base.join(templated_path);
    if !dry_run {
        let mut file = match File::create(templated_path) {
            Ok(f) => f,
            Err(e) => {
                error!("Failed to create file '{}': {}", templated_pathname, e);
                return Err(SchabloneError::FileError);
            }
        };
        if let Err(e) = file.write_all(&content.into_bytes()) {
            error!("Failed to write into new file: {}", e);
            return Err(SchabloneError::FileError);
        }
    }

    Ok(())
}

/// Build the schablone.
///
/// Take an input folder and build the schablone to a target. Use the parameters for templating.
pub fn build_schablone(name: &str, target: &str, parameters: &str, parameters_file: &str, dry_run: bool) {
    println!("Creating target directory '{}'", target);
    if let Err(e) = fs::create_dir(target) {
        error!("Failed to create directory '{}': {}!", target, e);
        ::std::process::exit(1);
    }

    let mut context = parse_parameters_file(parameters_file);
    context.extend(parse_parameters(parameters));
    debug!("context: {:?}", context);
    let mut path: String = name.to_owned();
    path.push_str(&"/**/*".to_owned());
    info!("Parsing schablone from {}", path);
    let mut tera = match Tera::new(&path) {
        Ok(t) => t,
        Err(e) => {
            error!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };
    debug!("tera: {:?}", tera);

    let source_path = Path::new(name);
    let target_path = Path::new(target);
    if let Err(e) = process_directory(
        source_path,
        source_path,
        target_path,
        &process_file,
        &mut tera,
        &context,
        dry_run
    ) {
        error!("Failed to run schablone: {}", e);
    }
}
