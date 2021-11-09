#![warn(clippy::pedantic)]
#![feature(once_cell)]

use oxipng::{optimize_from_memory, Options};
use std::{
    error::Error,
    fs::{self, read_dir, OpenOptions},
    io::{self, Read, Write},
    lazy::SyncLazy,
    path::{Path, PathBuf},
    process::{self, Command},
};

static IN_DIR: SyncLazy<PathBuf> = SyncLazy::new(|| PathBuf::from("./data"));
static OUT_DIR: SyncLazy<PathBuf> = SyncLazy::new(|| PathBuf::from("./data-out"));

type Res<T> = Result<T, Box<dyn Error>>;

fn main() -> Res<()> {
    if !IN_DIR.exists() {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::NotFound,
            "input directory not found",
        )));
    }

    if !OUT_DIR.exists() {
        fs::create_dir_all(&*OUT_DIR)?;
    }

    let read = read_dir(&*IN_DIR)?;

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

    items.iter().for_each(|x| match run_item(x) {
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

fn run_item(item: &Item) -> Res<()> {
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

    let mut out_path = OUT_DIR.clone();

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
