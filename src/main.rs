mod addon;
mod error;

fn main() -> error::Result<()> {
    let list = addon::list_installed(addon::Dir::Default)?;
    let a = list.first().unwrap();
    println!("{:#?}", a);
    let toc = a.get_toc()?;
    println!("{:#?}", toc);
    Ok(())
}
