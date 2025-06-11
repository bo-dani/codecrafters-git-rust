use anyhow::Context;
use flate2::read::ZlibDecoder;
use std::{
    ffi::CStr,
    fmt,
    io::{BufRead, BufReader, Read},
};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Blob,
    Tree,
    Commit,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Blob => write!(f, "blob"),
            Kind::Tree => write!(f, "tree"),
            Kind::Commit => write!(f, "commit"),
        }
    }
}

pub(crate) struct Object<R> {
    pub(crate) kind: Kind,
    pub(crate) expected_size: u64,
    pub(crate) reader: R,
}

impl Object<()> {
    pub(crate) fn read(hash: &str) -> anyhow::Result<Object<impl BufRead>> {
        let file = std::fs::File::open(format!(".git/objects/{}/{}", &hash[..2], &hash[2..]))
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
            "tree" => Kind::Tree,
            "commit" => Kind::Commit,
            _ => anyhow::bail!("unexpected object type {}", kind),
        };
        let size = size
            .parse::<u64>()
            .context("invalid size in .git/objects file header {size}")?;

        let d = d.take(size);
        Ok(Object {
            kind,
            expected_size: size,
            reader: d,
        })
    }
}
