use std::cmp;
use std::env;
use std::fs;
use std::io::{self, BufRead};
use std::path::PathBuf;
use std::process::ExitCode;

macro_rules! err_log {
    ($msg: expr) => (eprintln!("\x1b[31;1m> {}\x1b[0m", $msg));
    ($msg: expr, $($args:expr),+) => (eprintln!("\x1b[31;1m> {}\x1b[0m", format!($msg, $($args),+)));
}

macro_rules! inf_log {
    ($msg: expr) => (println!("> {}", $msg));
    ($msg: expr, $($args:expr),+) => (println!("> {}", format!($msg, $($args),+)));
}

#[derive(Debug, Default)]
struct Grog {
    query: String,
    haystacks: Vec<PathBuf>,
    ignored_exts: Vec<String>,
    verbose: bool,

    ignore_case: bool,
    recursive: bool,

    no_colors: bool,
}

fn usage(program_path: &PathBuf) {
    println!("Usage: {} [FLAGS] <query>", program_path.display());
    println!("Example:  {} -ri0 jajaja", program_path.display());
    println!("    <query>    - Query string to look for");
    println!("    -v         - Flag to add some logging");
    println!("    -i         - Flag to ignore casing when searching");
    println!("    -r         - Flag to recursively search subdirectories for query");
    println!("    -0         - Flag for removing coloring of matches");
    println!("    -f         - Search through file names instead of file contents");
    println!("    -h         - Displays this help message");
}

fn main() -> ExitCode {
    let mut args = env::args_os();
    let program_path = PathBuf::from(
        args.next()
            .expect("Program path should always be accesible as the first argument."),
    );
    let current_dir = match env::current_dir() {
        Ok(_) => PathBuf::from(if cfg!(windows) { ".\\" } else { "./" }),
        Err(e) => {
            err_log!("Failed to see what's the current directory\n    {}", e);
            return ExitCode::FAILURE;
        }
    };
    let mut i = 0;
    let mut search_file_names = false;
    let mut query = None;
    let mut grog = Grog::default();
    grog.ignored_exts.push("bin".to_string());
    grog.ignored_exts.push("exe".to_string());
    grog.ignored_exts.push("obj".to_string());
    grog.ignored_exts.push("o".to_string());
    grog.ignored_exts.push("dll".to_string());
    while let Some(arg) = args.next() {
        i += 1;
        if arg == "-h" || arg == "--help" || arg == "/?" {
            usage(&program_path);
            return ExitCode::SUCCESS;
        }

        {
            let arg = arg.to_string_lossy().to_string();
            // TODO: Maybe write this in a better way?
            if arg.starts_with("-") {
                for ch in arg.chars().skip(1) {
                    match ch {
                        'v' => grog.verbose = true,
                        'i' => grog.ignore_case = true,
                        'r' => grog.recursive = true,
                        'f' => search_file_names = true,
                        '0' => grog.no_colors = true,
                        _ => {
                            err_log!("Unknown flag -{} in arg {}", ch, arg);
                            return ExitCode::FAILURE;
                        }
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

    if query.is_none() {
        err_log!("Must pass a search query as an argument!");
        usage(&program_path);
        return ExitCode::FAILURE;
    }
    grog.query = match query.unwrap().into_string() {
        Ok(q) => {
            if grog.ignore_case {
                q.to_lowercase()
            } else {
                q
            }
        }
        Err(_) => {
            err_log!("Unsupported query string, must be valid Unicode");
            return ExitCode::FAILURE;
        }
    };
    if grog.query.is_empty() {
        err_log!("Query is empty, you cheeky bastard");
        return ExitCode::FAILURE;
    }

    if grog.verbose {
        inf_log!("Received {} arguments", i);
        inf_log!("Ignore casing: {}", grog.ignore_case);
        inf_log!("No coloring: {}", grog.no_colors);
        inf_log!("Querying: {:?}", grog.query);
        inf_log!("Query len: {}", grog.query.len());
    }
    if grog.haystacks.is_empty() {
        grog.haystacks.push(current_dir);
    }

    let mut read_dirs = 0usize;
    let search_for_query = if search_file_names {
        search_for_query_in_file_names
    } else {
        search_for_query_in_file_contents
    };
    while !grog.haystacks.is_empty() {
        if let Some(err) = search_for_query(&mut grog) {
            err_log!("{}", err);
        } else {
            read_dirs += 1;
        }
    }
    if read_dirs == 0 {
        return ExitCode::FAILURE;
    }

    return ExitCode::SUCCESS;
}

#[derive(Debug)]
struct DirReadFailed {
    error: std::io::Error,
    dir: PathBuf,
}
impl std::fmt::Display for DirReadFailed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "Failed to read dir `{}`: {}",
            self.dir.display(),
            self.error
        )
    }
}
impl std::error::Error for DirReadFailed {}

// Technically the correct or idiomatic rust would be to return Result<(), DirReadFailed> but do I really need that? No, if you said yes you're blind
fn search_for_query_in_file_contents(grog: &mut Grog) -> Option<DirReadFailed> {
    let dir = grog.haystacks.remove(0);
    let dir_entries: Vec<_> = match dir.read_dir() {
        Ok(x) => x.collect(),
        Err(e) => {
            let error = DirReadFailed { error: e, dir: dir };
            return Some(error);
        }
    };
    if grog.verbose {
        let entry_count = dir_entries.len();
        inf_log!("Dir Entries Count: {}", entry_count);
    }
    const START_PADDING: isize = 20isize;
    const END_PADDING: usize = 35usize;
    let query_len = grog.query.len();

    let mut subdirs = Vec::new();

    for entry_result in dir_entries {
        if let Ok(entry) = entry_result {
            let entry_path = entry.path();
            if !entry_path.is_file() {
                if entry_path.is_dir() {
                    // We should always have a filename but if we don't we don't uhhh, move on with life
                    if let Some(file_name) = entry_path.file_name() {
                        let file_name = file_name.to_string_lossy().to_string();
                        if file_name.starts_with(".") {
                            if grog.verbose {
                                inf_log!("Ignoring dot folder: {}", entry_path.display());
                            }
                        } else if grog.recursive {
                            subdirs.push(entry_path);
                        }
                    }
                } else if grog.verbose {
                    inf_log!("Skipping non file entry: {}", entry_path.display());
                }
                continue;
            }
            if let Some(extension) = entry_path.extension() {
                if grog.verbose {
                    let file_name = entry_path.file_name().unwrap().to_string_lossy();
                    inf_log!(
                        "> Skipping file with ${} extention: {}",
                        extension.to_string_lossy(),
                        file_name
                    );
                }
                continue;
            }
            let f = match fs::File::open(&entry_path) {
                Ok(f) => f,
                Err(e) => {
                    if grog.verbose {
                        err_log!("> Error when opening file: {}", e);
                    }
                    continue;
                }
            };
            let f = io::BufReader::new(f);
            let mut y = 0;
            for line in f.lines() {
                y += 1;
                if let Err(e) = line {
                    if grog.verbose {
                        err_log!("Failed to read line {}: {}", y, e);
                    }
                    continue;
                }
                let line = if grog.ignore_case {
                    line.unwrap().to_lowercase()
                } else {
                    line.unwrap()
                };
                let line_len = line.len();
                let mut lcpy = line.clone();
                let mut j = 0usize;
                while let Some(x) = lcpy.find(&grog.query) {
                    lcpy.drain(..(x + query_len));
                    let x = if j == 0 { j + x } else { j + x };
                    j = cmp::min(x + query_len, line_len);
                    let x = x as isize;
                    let i = cmp::max(0isize, x - START_PADDING) as usize;
                    let x = x as usize;
                    let preface = &line[i..x];
                    let preface = if i > 0 {
                        format!("...{}", preface)
                    } else {
                        String::from(preface)
                    };
                    let i = x + query_len;
                    let showcase = &line[x..i];
                    let x = i;
                    let i = cmp::min(x + query_len + END_PADDING, line_len);
                    let posface = &line[x..i];
                    let posface = if i < line.len() {
                        format!("{}...", posface)
                    } else {
                        String::from(posface)
                    };
                    let x = x + 1;
                    // &line[0..x];
                    let displayed = if grog.no_colors {
                        format!("{}{}{}", preface, showcase, posface)
                    } else {
                        format!("{}\x1b[36;1m{}\x1b[0m{}", preface, showcase, posface)
                    };
                    println!("{}:{}:{}> {}", entry_path.display(), y, x, displayed);
                }
            }
        }
    }

    grog.haystacks.append(&mut subdirs);

    return None;
}

fn search_for_query_in_file_names(grog: &mut Grog) -> Option<DirReadFailed> {
    let dir = grog.haystacks.remove(0);
    let dir_entries: Vec<_> = match dir.read_dir() {
        Ok(x) => x.collect(),
        Err(e) => {
            let error = DirReadFailed { error: e, dir: dir };
            return Some(error);
        }
    };
    if grog.verbose {
        let entry_count = dir_entries.len();
        inf_log!("Dir Entries Count: {}", entry_count);
    }
    const START_PADDING: isize = 20isize;
    const END_PADDING: usize = 35usize;
    let query_len = grog.query.len();

    let mut subdirs = Vec::new();

    for entry_result in dir_entries {
        if let Ok(entry) = entry_result {
            let entry_path = entry.path();
            // We should always have a filename but if we don't we don't uhhh, move on with life
            let file_name = entry_path.file_name();
            if file_name.is_none() {
                if grog.verbose {
                    err_log!("Hit a DirEntry with a path of `..`");
                }
                continue;
            }
            let file_name = file_name
                .expect("DirEntry should never be `..`")
                .to_string_lossy()
                .to_string();
            let is_dir = entry_path.is_dir();
            if is_dir {
                if file_name.starts_with(".") {
                    if grog.verbose {
                        inf_log!("Ignoring dot folder: {}", entry_path.display());
                    }
                } else if grog.recursive {
                    subdirs.push(entry_path.clone());
                }
            }
            if !file_name.contains(&grog.query) {
                continue;
            }
            let line = format!("{}", entry_path.display());
            let line_len = line.len();
            let mut lcpy = line.clone();
            let mut j = 0usize;
            while let Some(x) = lcpy.find(&grog.query) {
                lcpy.drain(..(x + query_len));
                let x = if j == 0 { j + x } else { j + x };
                j = cmp::min(x + query_len, line_len);
                let x = x as isize;
                let i = cmp::max(0isize, x - START_PADDING) as usize;
                let x = x as usize;
                let preface = String::from(&line[i..x]);
                let i = x + query_len;
                let showcase = &line[x..i];
                let x = i;
                let i = cmp::min(x + query_len + END_PADDING, line_len);
                let posface = String::from(&line[x..i]);
                let x = x + 1;
                // &line[0..x];
                let displayed = if grog.no_colors {
                    format!("{}{}{}", preface, showcase, posface)
                } else {
                    format!("{}\x1b[36;1m{}\x1b[0m{}", preface, showcase, posface)
                };
                println!("{}> {}", x, displayed);
            }
        }
    }

    grog.haystacks.append(&mut subdirs);

    return None;
}
