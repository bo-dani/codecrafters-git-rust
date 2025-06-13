use crate::objects::{Kind, Object};
use anyhow::Context;
use std::io::{Read, Write};
use std::{ffi::CStr, io::BufRead};

pub(crate) fn invoke(name_only: bool, tree_hash: String) -> anyhow::Result<()> {
    anyhow::ensure!(name_only, "we only support name_only");

    let mut object = Object::read(&tree_hash)?;
    let mut buf = Vec::new();
    let mut hash_buf = [0; 20];
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    loop {
        buf.clear();
        let n = object
            .reader
            .read_until(0, &mut buf)
            .context("could not read .git/object tree object {tree_hash}")?;
        if n == 0 {
            break;
        }
        let hash = object
            .reader
            .read_exact(&mut hash_buf)
            .context("failed to read sha1 from tree object")?;

        let mode_and_name = CStr::from_bytes_with_nul(&buf).context("failed to read tree entry")?;
        let mut mode_and_name = mode_and_name.to_bytes().splitn(2, |&b| b == b' ');
        let mode = mode_and_name.next().expect("split always yields one");
        let name = mode_and_name
            .next()
            .ok_or_else(|| anyhow::anyhow!("tree entry has no file name"))?;
        stdout
            .write_all(name)
            .context("failed to write name to stdout")?;
        writeln!(stdout, "").context("failed to write newline")?;
    }
    Ok(())
}
