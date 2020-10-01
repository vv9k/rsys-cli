use rsys::Result;

fn main() -> Result<()> {
    println!("{}", rsys::linux::hostname()?);

    Ok(())
}
