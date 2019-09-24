use source_error::Error;

fn main() {
    println!(
        "{}",
        Error::new(
            "whoopsie!",
            "../tests/source.json",
            include_str!("../tests/source.json").lines(),
            (3, 4)
        )
    )
}
