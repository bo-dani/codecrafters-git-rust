use anyhow::Context;

pub fn invoke() -> anyhow::Result<()> {
    std::fs::create_dir(".git").unwrap();
    std::fs::create_dir(".git/objects").unwrap();
    std::fs::create_dir(".git/refs").unwrap();
    std::fs::write(".git/HEAD", "ref: refs/heads/main\n").context("failed to create .git/HEAD")?;
    println!("Initialized git directory");
    Ok(())
}
