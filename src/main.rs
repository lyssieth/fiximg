#![warn(clippy::pedantic)]

use clap::{crate_authors, crate_version, App, Arg};
use oxipng::{optimize_from_memory, Options};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
    error::Error,
    fs::{read_dir, rename, OpenOptions},
    io::{Read, Write},
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
    rename: bool,
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
        .override_help("An image optimization commandline utility.")
        .arg(
            Arg::new("input")
                .help("The input directory")
                .forbid_empty_values(true)
                .required(true)
                .takes_value(true)
                .multiple_values(false)
                .multiple_occurrences(false)
                .validator(util::is_dir),
        )
        .arg(
            Arg::new("output")
                .help("The output directory")
                .forbid_empty_values(true)
                .required_unless_present("rename-in-place")
                .takes_value(true)
                .multiple_values(false)
                .multiple_occurrences(false)
                .validator(util::is_dir),
        )
        .arg(
            Arg::new("rename-in-place")
                .help("Rename the input files in place")
                .long("rename-in-place")
                .takes_value(false)
                .multiple_occurrences(false),
        )
        .get_matches();

    let cfg = {
        let rename_in_place = app.is_present("rename-in-place");

        Config {
            in_dir: app.value_of("input").unwrap().into(),
            out_dir: if rename_in_place {
                app.value_of("input").unwrap().into()
            } else {
                app.value_of("output").unwrap().into()
            },
            rename: rename_in_place,
            jpegoptim,
        }
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

    items.par_iter().for_each(|x| match run_item(x, &cfg) {
        Ok(_) => {}
        Err(e) => {
            println!("{:?}: {}", x.path, e);
        }
    });

    Ok(())
}

fn run_item(item: &Item, cfg: &Config) -> Res<()> {
    println!("Doing {:?}", item.path);
    let mut out_path = cfg.out_dir.clone();
    let buf = match item.file_type {
        FileType::Png => run_png(&item.path),
        FileType::Jpeg => run_jpeg(&item.path, &cfg.jpegoptim),
        FileType::Other => run_other(&item.path),
    }?;

    let hash = blake3::hash(&buf).to_hex();
    let ext = item
        .path
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or_default();

    out_path.push(format!("{}.{}", hash, ext));

    if cfg.rename {
        rename(&item.path, &out_path)?;
    } else {
        let mut f = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(out_path)?;

        f.write_all(&buf)?;
    }

    Ok(())
}

fn run_png(path: &Path) -> Res<Vec<u8>> {
    let mut buf = Vec::new();
    let mut data = OpenOptions::new().read(true).open(&path)?;
    data.read_to_end(&mut buf)?;

    let res = optimize_from_memory(&buf, &Options::default())?;

    Ok(res)
}

fn run_jpeg(path: &Path, jpegoptim: &str) -> Res<Vec<u8>> {
    let mut buf = Vec::new();
    let mut data = OpenOptions::new().read(true).open(&path)?;
    data.read_to_end(&mut buf)?;

    let mut cmd = Command::new(jpegoptim)
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
