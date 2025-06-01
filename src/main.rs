use anyhow::Context;
use clap::{Parser, Subcommand};
use flate2::read::ZlibDecoder;
use std::{
    ffi::CStr,
    fs,
    io::{BufRead, BufReader, Read, Write},
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Init,
    CatFile {
        #[clap(short = 'p')]
        pretty_print: bool,

        object_hash: String,
    },
}

#[derive(Debug)]
enum Kind {
    Blob,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.command {
        Command::Init => {
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
            println!("Initialized git directory");
        }
        Command::CatFile {
            pretty_print,
            object_hash,
        } => {
            anyhow::ensure!(
                pretty_print,
                "-p must be provided until 'mode' is supported"
            );

            let file = std::fs::File::open(format!(
                ".git/objects/{}/{}",
                &object_hash[..2],
                &object_hash[2..]
            ))
            .context("open file in .git/objects")?;
            let d = ZlibDecoder::new(file);
            let mut d = BufReader::new(d);
            let mut buf = Vec::new();
            d.read_until(0, &mut buf)
                .context("read header from .git/objects")?;

            let header = CStr::from_bytes_with_nul(&buf)
                .expect("the string is guaranteed to contain one nul at the end");
            let header = header
                .to_str()
                .context(".git/objects file header is not valid UTF-8")?;

            let Some((kind, size)) = header.split_once(' ') else {
                anyhow::bail!("unexpected .git/objects file header");
            };

            let kind = match kind {
                "blob" => Kind::Blob,
                _ => anyhow::bail!("unimplemented file type"),
            };
            let size = size
                .parse::<u64>()
                .context("invalid size in .git/objects file header {size}")?;

            let mut d = d.take(size);
            match kind {
                Kind::Blob => {
                    let stdout = std::io::stdout();
                    let mut stdout = stdout.lock();
                    std::io::copy(&mut d, &mut stdout)
                        .context("failed to write .git/objects content to stdout")?;
                }
            }
        }
    }

    Ok(())
}
