use clap::{Parser, Subcommand};
use pktools::{extract, insert, inspect};

#[derive(Parser)]
struct Opts {
    #[command(subcommand)]
    tool: ToolOpts,
}

#[derive(Subcommand)]
enum ToolOpts {
    Extract(extract::Opts),
    Insert(insert::Opts),
    Inspect(inspect::Opts),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let opts = Opts::parse();
    match opts.tool {
        ToolOpts::Extract(opts) => extract::run(opts),
        ToolOpts::Insert(opts) => insert::run(opts),
        ToolOpts::Inspect(opts) => inspect::run(opts),
    }
}
