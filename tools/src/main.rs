#[cfg(feature = "extra")]
mod info;
mod update;

mod data;
mod error;
mod job_crawler;
mod repos;
mod utils;

use clap::Parser;
use std::{fs, path::PathBuf, process};

use data::{Companies, Schema};
use repos::subtree;
use utils::{logger::Logger, text_file::TextFile};

pub type DynError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T = (), E = DynError> = std::result::Result<T, E>;

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Tools for maintaining the Awesome Software Companies in Bangladesh repo."
)]
struct Cli {
    #[arg(default_value = ".")]
    dir: PathBuf,

    #[arg(long)]
    backup: bool,

    /// Pull updates from upstream repos
    #[arg(long)]
    pull: bool,

    #[arg(long)]
    update: bool,

    /// Format data
    #[arg(long, short)]
    fmt: bool,

    #[arg(long, short)]
    docs: bool,

    #[arg(long, short)]
    crawl: bool,

    #[cfg(feature = "extra")]
    #[command(subcommand)]
    command: Option<Command>,
}

#[cfg(feature = "extra")]
#[derive(clap::Subcommand, Debug)]
enum Command {
    /// Fetch websites and extract `schema.org` data.
    Fetch {
        /// Re-fetch even if cached.
        #[arg(long)]
        force: bool,
    },
}

fn main() {
    Logger::init();

    if let Err(error) = cli() {
        log::error!("{error}");
    }

    let warnings = Logger::count_warnings();
    if warnings > 0 {
        eprintln!("::warning:: Found {warnings} warnings");
    }

    if Logger::has_error() {
        process::exit(1);
    }
}

fn cli() -> Result {
    let Cli {
        dir,
        backup,
        pull,
        mut update,
        mut fmt,
        mut docs,
        crawl,

        #[cfg(feature = "extra")]
        command,
    } = Cli::parse();

    let schema_file = TextFile::read(dir.join("./data/schema.toml"))?;
    let schema = Schema::parse(&schema_file.text)?;

    let companies_file = TextFile::read(dir.join("./data/companies.toml"))?;
    let mut companies = Companies::parse(&companies_file.text)?;

    companies.check_known_company_type(&schema);
    companies.check_no_redundant_technologies(&schema);

    if Logger::has_error() {
        return Ok(());
    }

    if backup {
        fs::create_dir_all(dir.join("./backup"))?;
        fs::write(dir.join("./backup/companies.toml"), companies.to_toml()?)?;
    }

    if pull {
        subtree::pull_repos(&dir)?;
        update = true;
    }

    if update {
        update::repos(&schema, &mut companies, &dir)?;
        fmt = true;
        docs = true;
    }

    #[cfg(feature = "extra")]
    if let Some(Command::Fetch { force }) = command {
        if force {
            log::info!("Clearing cache...");
            utils::fetch::clear_cache_dir()?;
        }
        info::fetch_info(&companies, &dir)?;
    }

    if crawl {
        job_crawler::main()?;
    }

    if fmt {
        companies_file.write(companies.to_toml()?)?;
    }

    if docs {
        TextFile::read(dir.join("./README.md"))?.write(companies.to_string())?;
    }

    Ok(())
}
