mod addon;
mod error;

fn main() -> error::Result<()> {
    for addon in addon::list_installed(addon::Dir::Default)? {
        println!("{:?}", addon);
    }
    Ok(())
}
