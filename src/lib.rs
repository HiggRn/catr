use clap::{arg, command};
use std::char;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

type RunResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    number_lines: bool,
    number_nonblank_lines: bool,
    show_tabs: bool,
    show_ends: bool,
    show_nonprinting: bool,
    squeeze_blank: bool
}

pub fn run(config: Config) -> RunResult<()> {
    for filename in config.files {
        let file = match open(&filename) {
            Ok(file) => file,            
            Err(err) => {
                eprintln!("{filename}: {err}");
                continue;
            }
        };

        let mut last_num = 0;
        let mut blank_appear: bool = false;

        for (line_num, line_result) in file.lines().enumerate() {
            let mut line = line_result?;
            if config.squeeze_blank && line.is_empty() {
                if blank_appear {
                    continue;
                } else {
                    blank_appear = true;
                }
            }

            if config.show_tabs {
                line = line.replace("\t", "^I");
            }

            if config.show_ends {
                line.push_str("$");
            }

            if config.show_nonprinting {
                line = line.chars().map(
                    |c| match c {
                        '\x01'..='\x1E' =>
                            "^".to_string() + 
                            &(char::from_u32(c as u32 + 0x40)
                                .unwrap()).to_string(),

                        '\x7F' => "^?".to_string(),

                        '\u{0080}'..='\u{00FF}'
                            => "M-".to_string() +
                                &(c as u32).to_string(),

                        _ => c.to_string()
                    }
                ).collect();
            }            

            if config.number_lines {
                println!("{:6}\t{line}", line_num + 1);
            } else if config.number_nonblank_lines {
                if line.is_empty() {
                    println!();
                } else {
                    last_num += 1;
                    println!("{:6}\t{line}", last_num);
                }
            } else {
                println!("{line}");
            }  
        }        
    }
    Ok(())
}

pub fn get_args() -> RunResult<Config> {
    let matches = command!()
        .args(&[
            arg!(files: [FILE] "Input file(s)")
                .num_args(0..)
                .default_value("-"),
            arg!(show_all: -A --"show-all" "equivalent to -vET"),
            arg!(number_nonblank: -b --"number-nonblank" "Number non-blank lines"),
            arg!(vE: -e "equivalent to -vE"),
            arg!(show_ends: -E --"show-ends" "display $ at end of each line"),
            arg!(number: -n --number "Number lines")
                .conflicts_with("number_nonblank"),
            arg!(squeeze_blank: -s --"squeeze-blank" "suppress repeated empty output lines"),
            arg!(vT: -t "equivalent to -vT"),
            arg!(show_tabs: -T --"show-tabs" "display TAB characters as ^I"),
            arg!(ignored: -u "(ignored)"),
            arg!(show_nonprinting: -v --"show-nonprinting" "use ^ and M- notation, except for LFD and TAB")
        ]) 
        .get_matches();
    
    let files = matches.get_many::<String>("files")
        .unwrap()
        .map(String::clone)
        .collect();
    
    let (show_all, vt, ve) = (
        matches.get_flag("show_all"),
        matches.get_flag("vT"),
        matches.get_flag("vE")
    );

    Ok(Config {
        files,
        number_lines: matches.get_flag("number"),
        number_nonblank_lines: matches.get_flag("number_nonblank"),
        show_tabs:
            matches.get_flag("show_tabs") ||
            show_all || vt,
        show_ends:
            matches.get_flag("show_ends") ||
            show_all || ve,
        show_nonprinting:
            matches.get_flag("show_nonprinting") ||
            show_all || ve || vt,
        squeeze_blank: matches.get_flag("squeeze_blank")
    })
}

pub fn open(filename: &str) -> RunResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?)))
    }
}
