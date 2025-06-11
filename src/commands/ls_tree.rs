use crate::objects::{Kind, Object};

pub(crate) fn invoke(name_only: bool, tree_ish: String) -> anyhow::Result<()> {
    let mut object = Object::read(&tree_ish)?;
    todo!();
    Ok(())
}
