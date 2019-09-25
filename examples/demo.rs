use source_error::{from_file, Position};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!(
        "{}",
        from_file("whoopsie!", "tests/source.json", Position::new(3, 4))?
    );
    Ok(())
}
