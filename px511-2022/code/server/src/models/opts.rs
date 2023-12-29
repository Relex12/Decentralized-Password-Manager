use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct Opts {
    #[clap(short, long)]
    pub debug: bool,

    #[clap(short, long)]
    pub address: String,

    #[clap(short, long)]
    pub port: u16,

    #[clap(short, long)]
    pub cert: PathBuf,

    #[clap(short, long)]
    pub key: PathBuf,
}
