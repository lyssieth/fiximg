#![warn(clippy::pedantic)]
#![feature(once_cell)]

use clap::{crate_authors, crate_version, App, Arg};
use oxipng::{optimize_from_memory, Options};
use std::{
    error::Error,
    fs::{self, read_dir, OpenOptions},
    io::{self, Read, Write},
    lazy::SyncLazy,
    path::{Path, PathBuf},
    process::{self, Command},
};
use util::check_jpegoptim;

type Res<T> = Result<T, Box<dyn Error>>;

mod util;

#[derive(Debug, Clone)]
struct Config {
    in_dir: PathBuf,
    out_dir: PathBuf,
    rename_in_place: bool,
    jpegoptim: String,
}

fn main() -> Res<()> {
    let jpegoptim = match check_jpegoptim() {
        util::CheckResult::Ok(path) => path,
        util::CheckResult::NotFound => {
            #[cfg(unix)]
            eprintln!("jpegoptim not found, please ensure there is a jpegoptim on the PATH");
            #[cfg(target_os = "windows")]
            eprintln!(
                "jpegoptim.exe not found, please ensure there is a jpegoptim.exe on the PATH"
            );
            std::process::exit(1);
        }
    };

    let app = App::new("fiximg")
        .author(crate_authors!("\n"))
        .version(crate_version!())
        .about("An image optimization commandline utility.")
        .arg(
            Arg::new("input")
                .about("The input directory")
                .forbid_empty_values(true)
                .required(true)
                .takes_value(true)
                .multiple_values(false)
                .multiple_occurrences(false)
                .validator(util::is_dir),
        )
        .arg(
            Arg::new("output")
                .about("The output directory")
                .forbid_empty_values(true)
                .required_unless_present("rename_in_place")
                .takes_value(true)
                .multiple_values(false)
                .multiple_occurrences(false)
                .validator(util::is_dir),
        )
        .arg(
            Arg::new("rename_in_place")
                .about("Rename the input files in place")
                .takes_value(false)
                .multiple_occurrences(false),
        )
        .get_matches();

    let cfg = Config {
        in_dir: app.value_of("input").unwrap().into(),
        out_dir: app.value_of("output").unwrap().into(),
        rename_in_place: app.is_present("rename_in_place"),
        jpegoptim,
    };

    let read = read_dir(&cfg.in_dir)?;

    let mut items = Vec::new();

    read.for_each(|x| {
        if let Ok(entry) = x {
            items.push(Item {
                path: entry.path(),
                file_type: match entry
                    .path()
                    .extension()
                    .and_then(std::ffi::OsStr::to_str)
                    .unwrap_or_default()
                {
                    "png" => FileType::Png,
                    "jpeg" | "jpg" => FileType::Jpeg,
                    _ => FileType::Other,
                },
            });
        }
    });

    let mut queue = Vec::new();

    items
        .iter()
        .for_each(|x| match run_item(x, cfg.out_dir.clone()) {
            Ok(_) => {}
            Err(e) => {
                queue.push(format!("{:?}: {}", x.path, e));
            }
        });

    for x in queue {
        println!("{}", x);
    }

    Ok(())
}

fn run_item(item: &Item, mut out_path: PathBuf) -> Res<()> {
    println!("Doing {:?}", item.path);
    let buf = match item.file_type {
        FileType::Png => run_png(&item.path),
        FileType::Jpeg => run_jpeg(&item.path),
        FileType::Other => run_other(&item.path),
    }?;

    let hash = blake3::hash(&buf).to_hex();
    let ext = item
        .path
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or_default();

    out_path.push(format!("{}.{}", hash, ext));

    let mut f = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(out_path)?;

    f.write_all(&buf)?;

    Ok(())
}

fn run_png(path: &Path) -> Res<Vec<u8>> {
    let mut buf = Vec::new();
    let mut data = OpenOptions::new().read(true).open(&path)?;
    data.read_to_end(&mut buf)?;

    let res = optimize_from_memory(&buf, &Options::default())?;

    Ok(res)
}

fn run_jpeg(path: &Path) -> Res<Vec<u8>> {
    let mut buf = Vec::new();
    let mut data = OpenOptions::new().read(true).open(&path)?;
    data.read_to_end(&mut buf)?;

    let mut cmd = Command::new("jpegoptim")
        .arg("--stdin")
        .arg("--stdout")
        .stdin(process::Stdio::piped())
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::null())
        .spawn()?;

    cmd.stdin.as_mut().unwrap().write_all(&buf)?;

    let res = cmd.wait_with_output()?;

    Ok(res.stdout)
}

fn run_other(path: &Path) -> Res<Vec<u8>> {
    let mut buf = Vec::new();
    let mut data = OpenOptions::new().read(true).open(&path)?;
    data.read_to_end(&mut buf)?;

    Ok(buf)
}

#[derive(Debug, Clone)]
struct Item {
    path: PathBuf,
    file_type: FileType,
}

#[derive(Debug, Copy, Clone)]
enum FileType {
    Png,
    Jpeg,
    Other,
}
