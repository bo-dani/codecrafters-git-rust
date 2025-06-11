use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;
mod objects;

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
    LsTree {
        #[clap(long)]
        name_only: bool,

        tree_ish: String,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.command {
        Command::Init => {
            commands::init::invoke()?;
        }
        Command::CatFile {
            pretty_print,
            object_hash,
        } => {
            commands::cat_file::invoke(&object_hash, pretty_print)?;
        }
        Command::HashObject { write, file } => {
            commands::hash_object::invoke(&file, write)?;
        }
        Command::LsTree {
            name_only,
            tree_ish,
        } => {
            commands::ls_tree::invoke(name_only, tree_ish)?;
        }
    }

    Ok(())
}
