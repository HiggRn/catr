use clap::{App, Arg};
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
    let matches = App::new("catr")
        .version("0.2.0")
        .about("cat in Rust")
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .help("Input file(s)")
                .multiple(true)
                .default_value("-")
        )
        .arg(
            Arg::with_name("show_all")
                .short("A")
                .long("show-all")
                .help("equivalent to -vET")
                .takes_value(false)
                .display_order(1)
        )
        .arg(
            Arg::with_name("number_nonblank")
                .short("b")
                .long("number-nonblank")
                .help("Number non-blank lines")
                .takes_value(false)
                .display_order(2)
        )      
        .arg(
            Arg::with_name("vE")
                .short("e")
                .help("equivalent to -vE")
                .takes_value(false)
                .display_order(3)
        )
        .arg(
            Arg::with_name("show_ends")
                .short("E")
                .long("show-ends")
                .help("display $ at end of each line")
                .takes_value(false)
                .display_order(4)
        )
        .arg(
            Arg::with_name("number")
                .short("n")
                .long("number")
                .help("Number lines")
                .takes_value(false)
                .conflicts_with("number_nonblank")
                .display_order(5)
        )
        .arg(
            Arg::with_name("squeeze_blank")
                .short("s")
                .long("--squeeze-blank")
                .help("suppress repeated empty output lines")
                .takes_value(false)
                .display_order(6)
        )
        .arg(
            Arg::with_name("vT")
                .short("t")
                .help("equivalent to -vT")
                .takes_value(false)
                .display_order(7)
        )
        .arg(
            Arg::with_name("show_tabs")
                .short("T")
                .long("show-tabs")
                .help("display TAB characters as ^I")
                .takes_value(false)
                .display_order(8)
        )
        .arg(
            Arg::with_name("ignored")
                .short("u")
                .help("(ignored)")
                .takes_value(false)
                .display_order(9)
        )
        .arg(
            Arg::with_name("show_nonprinting")
                .short("v")
                .long("show-nonprinting")
                .help("use ^ and M- notation, except for LFD and TAB")
                .takes_value(false)
                .display_order(10)
        )
        .get_matches();
    
    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        number_lines: matches.is_present("number"),
        number_nonblank_lines: matches.is_present("number_nonblank"),
        show_tabs:
            matches.is_present("show_tabs") ||
            matches.is_present("show_all") ||
            matches.is_present("vT"),
        show_ends:
            matches.is_present("show_ends") ||
            matches.is_present("show_all") ||
            matches.is_present("vE"),
        show_nonprinting:
            matches.is_present("show_nonprinting") ||
            matches.is_present("show_all") ||
            matches.is_present("vE") ||
            matches.is_present("vT"),
        squeeze_blank: matches.is_present("squeeze_blank")
    })
}

pub fn open(filename: &str) -> RunResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?)))
    }
}
