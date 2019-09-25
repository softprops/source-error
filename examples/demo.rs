use source_error::from_file;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("{}", from_file("whoopsie!", "tests/source.json", (3, 4))?);
    Ok(())
}
