#[macro_use] extern crate clap;
extern crate dirs;
extern crate reqwest;
extern crate tokio;
extern crate criterion;
extern crate itertools;
extern crate num;
extern crate nom;
extern crate termion;
extern crate rayon;

mod solutions;

use clap::App;
use failure::Error;
use std::path::PathBuf;
use criterion::Criterion;

fn get_session() -> Result<String, Error> {
    let home = dirs::home_dir().unwrap();
    let session_file = home.join(".aoc");
    std::fs::read_to_string(session_file).map_err(From::from)
}

async fn download_input(year: &str, day: &str) -> Result<(), Error> {
    let session = get_session()?;
    let client = reqwest::Client::new();
    let url = format!("https://adventofcode.com/{}/day/{}/input", year, day);
    let res =
        client
            .get(&url)
            .header("cookie", format!("session={}", session))
            .send()
        .await?.text().await?;
    std::fs::create_dir_all(format!("{}/{}", year, day))?;
    std::fs::write(get_input_location(year, day), res)?;
    Ok(())
}

fn get_input_location(year: &str, day: &str) -> PathBuf {
    PathBuf::from(format!("{}/{}/input", year, day))
}

async fn run_input(year: &str, day: &str, part: &str) -> Result<(), Error> {
    let input_path =get_input_location(year, day);
    if !input_path.exists() {
        download_input(year, day).await?;
    }
    let input = std::fs::read_to_string(input_path)?;
    let res = solutions::run(year, day, part == "1", &input)?;
    println!("Result is {}", res);
    Ok(())
}

async fn bench(year: &str, day: &str, part: &str) -> Result<(), Error> {
    let input_path =get_input_location(year, day);
    if !input_path.exists() {
        download_input(year, day).await?;
    }
    let input = std::fs::read_to_string(input_path)?;
    let fn_name = format!("year {} day {} part {}", year, day, part);
    Criterion::default()
        .bench_function(&fn_name,|b| {
            b.iter(|| criterion::black_box(solutions::run(year, day, part == "1", &input).expect("working")))
        })
        .final_summary();
    Ok(())
}

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from(yaml).get_matches();
    let rt = tokio::runtime::Runtime::new().expect("runtime");
    if let Some(args) = matches.subcommand_matches("input") {
        let year = args.value_of("YEAR").expect("required");
        let day = args.value_of("DAY").expect("required");
        rt.block_on(download_input(year, day)).unwrap();
    }
    if let Some(args) = matches.subcommand_matches("run") {
        let year = args.value_of("YEAR").expect("required");
        let day = args.value_of("DAY").expect("required");
        let part = args.value_of("PART").unwrap_or("1");
        rt.block_on(run_input(year, day, part)).unwrap();
    }
    if let Some(args) = matches.subcommand_matches("bench") {
        let year = args.value_of("YEAR").expect("required");
        let day = args.value_of("DAY").expect("required");
        let part = args.value_of("PART").unwrap_or("1");
        rt.block_on(bench(year, day, part)).unwrap();
    }

}
