use anyhow::Context;
use clap::{Parser, Subcommand};
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use sha1::{Digest, Sha1};
use std::{
    ffi::CStr,
    fs,
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
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
    HashObject {
        #[clap(short = 'w')]
        write: bool,

        file: PathBuf,
    },
}

#[derive(Debug)]
enum Kind {
    Blob,
}

struct HashWriter<W> {
    writer: W,
    hasher: Sha1,
}

impl<W> Write for HashWriter<W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.writer.write(buf)?;
        self.hasher.update(&buf[..n]);
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

fn write_blob<W>(file: &Path, writer: W) -> anyhow::Result<String>
where
    W: Write,
{
    let writer = ZlibEncoder::new(writer, Compression::default());
    let mut writer = HashWriter {
        hasher: Sha1::new(),
        writer,
    };

    let stat = std::fs::metadata(file).with_context(|| format!("stat {}", file.display()))?;
    write!(writer, "blob ")?;
    write!(writer, "{}\0", stat.len())?;
    let mut file =
        std::fs::File::open(&file).with_context(|| format!("open {}", file.display()))?;
    std::io::copy(&mut file, &mut writer).context("stream file into encoder")?;
    let _ = writer.writer.finish()?;
    let hash = writer.hasher.finalize();

    Ok(hex::encode(hash))
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
        Command::HashObject { write, file } => {
            let hash = if write {
                let tmp = "tmp";
                let hash = write_blob(
                    &file,
                    std::fs::File::create(tmp).context("failed to create 'tmp' file")?,
                )
                .context("failed to write 'temp' file")?;
                std::fs::create_dir_all(format!(".git/objects/{}", &hash[..2]))
                    .context("failed to create .git/objects dir")?;
                std::fs::rename(tmp, format!(".git/objects/{}/{}", &hash[..2], &hash[2..]))
                    .context("failed to move 'tmp' file into .git/objects")?;
                hash
            } else {
                write_blob(&file, std::io::sink()).context("failed to write to 'sink'")?
            };

            println!("{hash}");
        }
    }

    Ok(())
}
