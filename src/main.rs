use mofmt::{ModelicaCST, SyntaxKind};
use std::io::{stdout, Write};
use std::path::{Path, PathBuf};
use std::{env, fs};

const VERSION: &str = "0.6.0";

const HELP: &str = r#"
mofmt: Modelica code formatter

Usage: mofmt [OPTIONS] <PATHS>

Options:
-h, --help: display this message and exit
-v, --version: display a version number and exit
--check: run mofmt in check mode (without modifying the file)
--line-length <N>: set maximum line length (disabled by default)
"#;

const EOL: &str = if cfg!(windows) { "\r\n" } else { "\n" };

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Missing PATHS arguments.\n{}", HELP);
        std::process::exit(1);
    } else if ["-h", "--help"].contains(&args[1].as_str()) {
        println!("{}", HELP);
        std::process::exit(0);
    } else if ["-v", "--version"].contains(&args[1].as_str()) {
        println!("mofmt, {}", VERSION);
        std::process::exit(0);
    } else {
        // Parse options
        let mut check = false;
        let mut line_length = None;
        let mut i = 1;

        while i < args.len() {
            if args[i] == "--check" {
                check = true;
                i += 1;
            } else if args[i] == "--line-length" {
                if i + 1 >= args.len() {
                    eprintln!("Missing value for --line-length argument.\n{}", HELP);
                    std::process::exit(1);
                }
                match args[i + 1].parse::<usize>() {
                    Ok(n) if n > 0 => line_length = Some(n),
                    _ => {
                        eprintln!("Invalid line length: '{}'. Must be a positive integer.\n{}", args[i + 1], HELP);
                        std::process::exit(1);
                    }
                }
                i += 2;
            } else if args[i].starts_with('-') {
                eprintln!("Unrecognized option: '{}'.\n{}", args[i], HELP);
                std::process::exit(1);
            } else {
                break;
            }
        }

        if i >= args.len() {
            eprintln!("Missing PATHS arguments.\n{}", HELP);
            std::process::exit(1);
        }

        format_files(&args[i..], check, line_length);
    }
}

/// Format files specified in the argument list
fn format_files(args: &[String], check: bool, line_length: Option<usize>) {
    let mut code = 0;
    let mut files = Vec::new();
    let mut lock = stdout().lock();
    args.iter()
        .map(PathBuf::from)
        .map(|p| {
            if p.is_dir() {
                get_files_from_dir(p)
            } else {
                vec![p]
            }
        })
        .for_each(|mut v| files.append(&mut v));
    files.iter().for_each(|p| {
        let contents = read_file(p);
        let name = p.display();
        match contents {
            Ok(source) => {
                let parsed = ModelicaCST::from(name.to_string(), source, SyntaxKind::StoredDefinition);
                let mut errors = parsed.tokens().errors();
                errors.append(&mut parsed.errors());
                if !errors.is_empty() {
                    writeln!(
                        lock,
                        "\n{}: \x1b[31msyntax errors detected\x1b[0m\n{}",
                        name,
                        errors.join("\n")
                    )
                    .unwrap();
                    code = 1;
                } else {
                    let output = match line_length {
                        Some(len) => parsed.pretty_print_with_line_length(len),
                        None => parsed.pretty_print(),
                    } + EOL;
                    if check {
                        if output != parsed.tokens().code() {
                            code = 1;
                            writeln!(lock, "{}: check failed", name).unwrap();
                        } else {
                            writeln!(lock, "{}: check passed", name).unwrap();
                        }
                    } else {
                        write_file(p, output);
                    }
                }
            }
            Err(e) => {
                eprintln!("{}: error: {}", name, e);
                code = 1;
            }
        }
    });
    std::process::exit(code);
}

/// Return all Modelica files from the given directory
fn get_files_from_dir(dir: PathBuf) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let paths = fs::read_dir(&dir)
        .unwrap_or_else(|_| panic!("{}: error reading from a directory", dir.display()));
    paths
        .map(|e| e.unwrap().path())
        .map(|p| {
            if p.is_dir() {
                get_files_from_dir(p)
            } else if is_modelica(p.as_path()) {
                vec![p]
            } else {
                Vec::new()
            }
        })
        .for_each(|mut v| files.append(&mut v));

    files
}

/// Return `true` if the file is a Modelica file
fn is_modelica(f: &Path) -> bool {
    if let Some(suffix) = f.extension() {
        return suffix == "mo";
    }
    false
}

/// Return contents of the Modelica file
fn read_file(from: &Path) -> Result<String, String> {
    if !is_modelica(from) {
        return Err(format!("{} is not a Modelica file", from.display()));
    }
    match fs::read_to_string(from) {
        Ok(s) => Ok(s),
        Err(e) => Err(e.to_string()),
    }
}

/// Write formatted code to a file
fn write_file(to: &Path, code: String) {
    fs::write(to, code).unwrap_or_else(|_| panic!("{}: error writing a file", to.display()));
}
