use anyhow::Context;
use flate2::{write::ZlibEncoder, Compression};
use sha1::{Digest, Sha1};
use std::io::Write;
use std::path::{Path, PathBuf};

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

pub fn invoke(file: &PathBuf, write: bool) -> anyhow::Result<()> {
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
    Ok(())
}
