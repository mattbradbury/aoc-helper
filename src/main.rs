use std::{error::Error, io::Write, path::PathBuf};

use chrono::{self, Datelike};
use clap::{App, Arg};
use ureq::AgentBuilder;

struct Props {
    day: String,
    year: String,
    cookie: String,
    output: PathBuf,
}

const HELP_EXTRA: &str = r#"Process exit codes:
 0 - Normal Exit
 1 - Cookie not set
 2 - Destination file already exists
 3 - Error connecting to AoC or Server error
 4 - Unable to write to output file

"#;

fn main() {
    let (day, year) = get_day_year();
    let mut config_dir = dirs::config_dir().unwrap();
    config_dir.push("aoc");

    let matches = App::new("Advent of Code Input Retriever")
        .version("0.1.1")
        .author("Matt Bradbury <matt@bexars.com>")
        .after_help(HELP_EXTRA)
        .arg(
            Arg::with_name("cookie")
                .short("c")
                .value_name("cookie")
                .help(&format!(
                    "Set AoC Cookie in: {}",
                    config_dir.to_str().unwrap()
                ))
                .takes_value(true),
        )
        .arg(
            Arg::with_name("day")
                .help("Which day to retrieve [1-25]")
                .index(1)
                .default_value(&day)
                .validator(is_day),
        )
        .arg(
            Arg::with_name("year")
                .help("Which year to retrieve [2015+]")
                .index(2)
                .default_value(&year)
                .validator(is_year),
        )
        .arg(
            Arg::with_name("output")
                .help("Relative directory to store retrieved input")
                .short("o")
                .default_value("input/"),
        )
        .get_matches();

    if let Some(cookie) = matches.value_of("cookie") {
        save_cookie(cookie);
        return;
    }

    config_dir.push("cookie"); // Actual file we read/write cookie from

    let cookie = get_cookie(config_dir);

    let mut output_dir = match std::env::current_dir() {
        Ok(o) => o,
        Err(_) => {
            println!("Unable to determine current working directory");
            std::process::exit(255);
        }
    };

    let rel_output = matches.value_of("output").unwrap_or("input");

    let day = matches.value_of("day").unwrap().to_owned();
    let year = matches.value_of("year").unwrap().to_owned();

    output_dir.push(rel_output);
    output_dir.push(format!("{}-{}.txt", &year, &day));

    let props = Props {
        day,
        year,
        cookie,
        output: output_dir,
    };

    if props.output.exists() {
        println!(
            "Destination file already exists: {} ",
            props.output.to_str().unwrap()
        );
        std::process::exit(2);
    }

    println!("Downloading to: {}", props.output.to_str().unwrap());
    let output = download_day(&props);

    match std::fs::write(props.output, output) {
        Ok(_) => println!("Success."),
        Err(e) => {
            println!("Unable to write to destination: {}", e.to_string());
            std::process::exit(4)
        }
    }
}

fn get_cookie(config_dir: PathBuf) -> String {
    match std::fs::read_to_string(config_dir) {
        Ok(c) => c,
        Err(e) => {
            println!(
                "Unable to load cookie.  Use -c to set it. [{}]",
                e.to_string()
            );
            std::process::exit(1);
        }
    }
}

fn is_day(day: String) -> Result<(), String> {
    let day: u32 = day
        .parse()
        .map_err(|_| format!("{} Must be a number", day))?;
    if !(1..=25).contains(&day) {
        return Err("Day must be between 1 and 25".to_owned());
    }
    Ok(())
}

fn is_year(year: String) -> Result<(), String> {
    let year: u32 = year.parse().map_err(|_| "Must be a number".to_owned())?;
    if year < 2015 {
        return Err("Year must be 2015 or greater".to_owned());
    }
    Ok(())
}

fn save_cookie(cookie: &str) {
    let cookie = cookie.trim();
    let mut config_file = dirs::config_dir().unwrap();
    config_file.push("aoc");
    let _res = std::fs::create_dir(&config_file);
    config_file.push("cookie");
    // println!("{:?}", config_file);
    let mut file = std::fs::File::create(config_file).unwrap();
    let perms = file.metadata().unwrap().permissions();
    file.set_permissions(perms).unwrap();
    file.write_all(cookie.as_bytes()).unwrap();
    println!("Cookie stored.");
}

fn get_day_year() -> (String, String) {
    // UNWRAP because it's a constant known good value
    let offset = chrono::FixedOffset::west_opt(5 * 3600).unwrap(); // EST (UTC -0500) which is AoC server TZ
    let time = chrono::Utc::now().with_timezone(&offset);
    (time.day().to_string(), time.year().to_string())
}

fn download_day(props: &Props) -> String {
    println!("Downloading day: {}", props.day);
    let response = match make_request(props) {
        Ok(resp) => resp,
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(3)
        }
    };
    // println!("{}", response);
    response
}

fn make_request(props: &Props) -> Result<String, Box<dyn Error>> {
    let header = format!("session={}", &props.cookie);

    let client = AgentBuilder::new().user_agent("aoc-helper 0.1 matt@bexars.com").build();

    let url = format!(
        "https://adventofcode.com/{}/day/{}/input",
        &props.year, &props.day
    );

    println!("Connecting to: {}", url);

    Ok(client
        .get(&url)
        .set("cookie", &header)
        .call()?
        .into_string()?)
}
