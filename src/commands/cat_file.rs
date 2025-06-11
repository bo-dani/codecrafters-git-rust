use crate::objects::{Kind, Object};
use anyhow::{Context, Ok};

pub(crate) fn invoke(object_hash: &str, pretty_print: bool) -> anyhow::Result<()> {
    anyhow::ensure!(
        pretty_print,
        "-p must be provided until 'mode' is supported"
    );

    let mut object = Object::read(object_hash)?;
    match object.kind {
        Kind::Blob => {
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();
            std::io::copy(&mut object.reader, &mut stdout)
                .context("failed to write .git/objects content to stdout")?;
        }
        Kind::Tree => {}
        Kind::Commit => {}
    }
    Ok(())
}
