use structopt::StructOpt;

mod addon;
mod error;

#[derive(Debug, StructOpt)]
#[structopt(name = "addons", about = "Simple addon manager for WoW")]
enum CLI {
    #[structopt(name = "list", about = "Lists installed addons")]
    ListInstalled,
}

fn main() -> Result<(), error::Error> {
    let app = CLI::from_args();
    match &app {
        CLI::ListInstalled => {
            let mut list = addon::list_installed(addon::Dir::Default)?;
            list.sort_unstable_by(|a, b| a.name.cmp(&b.name));
            for a in list {
                println!("{}", a);
            }
        }
    }
    Ok(())
}
