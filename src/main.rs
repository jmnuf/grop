use std::env;
use std::process::ExitCode;
use std::path::PathBuf;
use std::fs;
use std::io::{self, BufRead};
use std::cmp;

macro_rules! err_log {
    ($msg: expr) => (eprintln!("\x1b[31;1m> {}\x1b[0m", $msg));
    ($msg: expr, $($args:expr),+) => (eprintln!("\x1b[31;1m> {}\x1b[0m", format!($msg, $($args),+)));
}

macro_rules! inf_log {
    ($msg: expr) => (println!("> {}", $msg));
    ($msg: expr, $($args:expr),+) => (println!("> {}", format!($msg, $($args),+)));
}

fn usage(program_path: &PathBuf) {
    println!("Usage:");
    println!("  {} [-v] [-i] [-0] <query>", program_path.display());
    println!("  {} [-iv0] <query>", program_path.display());
    println!("    -v         - Optional: Add some logging");
    println!("    -i         - Optional: Ignore casing when searching");
    println!("    -0         - Optional: Remove coloring of match showcased");
    println!("    <query>    - Required: Query string to look for");
}

fn main() -> ExitCode {
    let mut args = env::args_os();
    let program_path = PathBuf::from(args.next().expect("Program path should always be accesible as the first argument."));
    let current_dir = match env::current_dir() {
	Ok(_) => PathBuf::from(if cfg!(windows) { ".\\" } else { "./" }),
	Err(e) => {
	    err_log!("Failed to see what's the current directory\n    {}", e);
	    return ExitCode::FAILURE;
	},
    };
    let mut i = 0;
    let mut query = None;
    let mut verbose = false;
    let mut ignore_case = false;
    let mut no_colors = false;
    while let Some(arg) = args.next() {
	i += 1;
	if arg == "-v" {
	    verbose = true;
	    continue;
	}
	if arg == "-i" {
	    ignore_case = true;
	    continue;
	}
	if arg == "-0" {
	    no_colors = true;
	    continue;
	}
	{
	    let arg = arg.to_string_lossy().to_string();
	    if arg.starts_with("-") {
		for ch in arg.chars().skip(1) {
		    match ch {
			'v' => verbose = true,
			'i' => ignore_case = true,
			'0' => no_colors = true,
			_ => {
			    err_log!("Unknown flag -{}", ch);
			    return ExitCode::FAILURE;
			},
		    }
		}
		continue;
	    }
	}
	if query.is_none() {
	    query = Some(arg);
	}
    }
    let i = i;
    let verbose = verbose;
    let ignore_case = ignore_case;
    let no_colors = no_colors;
    
    if query.is_none() {
	err_log!("Must pass a search query as an argument!");
	usage(&program_path);
	return ExitCode::FAILURE;
    }
    let query = query.unwrap();
    let query = match query.into_string() {
	Ok(q) => if ignore_case { q.to_lowercase() } else { q },
	Err(_) => {
	    err_log!("Unsupported query string, must be valid Unicode");
	    return ExitCode::FAILURE;
	},
    };
    let query_len = query.len();
    if verbose {
	inf_log!("Received {} arguments", i);
	inf_log!("Ignore casing: {}", ignore_case);
	inf_log!("No coloring: {}", no_colors);
	inf_log!("Querying: {:?}", query);
	inf_log!("Query len: {}", query_len);
    }

    let dir_entries:Vec<_> = match current_dir.read_dir() {
	Ok(x) => x.collect(),
	Err(e) => {
	    err_log!("Failed to read current dir\n    {}", e);
	    return ExitCode::FAILURE;
	}
    };
    if verbose {
	let entry_count = dir_entries.len();
	inf_log!("Dir Entries Count: {}", entry_count);
    }
    let start_padding = 20isize;
    let end_padding = 35usize;
    for entry_result in dir_entries {
	if let Ok(entry) = entry_result {
	    let entry_path = entry.path();
	    if ! entry_path.is_file() {
		if verbose {
		    inf_log!("Skipping non file entry: {}", entry_path.display());
		}
		continue;
	    }
	    if let Some(extension) = entry_path.extension() {
		if extension == "bin" {
		    if verbose {
			let file_name = entry_path.file_name().unwrap().to_string_lossy();
			inf_log!("> Skipping bin file (assuming binary): {}", file_name);
		    }
		    continue;
		}
	    }
	    let f = match fs::File::open(&entry_path) {
		Ok(f) => f,
		Err(e) => {
		    if verbose {
			err_log!("> Error when opening file: {}", e);
		    }
		    continue;
		},
	    };
	    let f = io::BufReader::new(f);
	    let mut y = 0;
	    for line in f.lines() {
		y += 1;
		if let Err(e) = line {
		    err_log!("Failed to read line {}: {}", y, e);
		    continue;
		}
		let line = if ignore_case {
		    line.unwrap().to_lowercase()
		} else {
		    line.unwrap()
		};
		if let Some(x) = line.find(&query) {
		    let x = x as isize;
		    let i = cmp::max(0isize, x - start_padding) as usize;
		    let x = x as usize;
		    let preface = &line[i..x];
		    let preface = if i > 0 { format!("...{}", preface) } else { String::from(preface) };
		    let i = x + query_len;
		    let showcase = &line[x..i];
		    let x = i;
		    let i = cmp::min(x + query_len + end_padding, line.len());
		    let posface = &line[x..i];
		    let posface = if i < line.len() { format!("{}...", posface) } else { String::from(posface) };
		    let x = x + 1;
		    // &line[0..x];
		    let displayed = if no_colors {
			format!("{}{}{}", preface, showcase, posface)
		    } else {
			format!("{}\x1b[36;1m{}\x1b[0m{}", preface, showcase, posface)
		    };
		    println!("{}:{}:{}> {}", entry_path.display(), y, x, displayed);
		}
	    }
	}
    }

    return ExitCode::SUCCESS;
}
