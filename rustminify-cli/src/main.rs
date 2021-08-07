use std::io::{self, Read as _, Write as _};

use anyhow::{anyhow, Context as _};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Opt {
    /// Removes documentation and `#[{warn, deny, forbid}(missing_docs)]`
    #[structopt(long)]
    remove_docs: bool,
}

fn main() -> anyhow::Result<()> {
    let Opt { remove_docs } = Opt::from_args();
    let mut code = parse(&read_from_stdin()?)?;
    if remove_docs {
        code = rustminify::remove_docs(code);
    }
    print(&rustminify::minify_file(&code))?;
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
