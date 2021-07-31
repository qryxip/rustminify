use std::io::{self, Read as _, Write as _};

use anyhow::{anyhow, Context as _};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Opt {}

fn main() -> anyhow::Result<()> {
    Opt::from_args();
    print(&rustminify::minify_file(&parse(&read_from_stdin()?)?))?;
    Ok(())
}

fn print(s: &str) -> io::Result<()> {
    io::stdout().write_all(s.as_ref())?;
    io::stdout().flush()
}

fn parse(s: &str) -> anyhow::Result<syn::File> {
    syn::parse_file(s).map_err(|e| anyhow!("could not parse the input: {}", e))
}

fn read_from_stdin() -> anyhow::Result<String> {
    let mut buf = "".to_owned();
    io::stdin()
        .read_to_string(&mut buf)
        .with_context(|| "could not read input")?;
    Ok(buf)
}
