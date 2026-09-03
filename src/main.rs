use laz_to_text::{user_interface};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    user_interface()?;
    Ok(())
}
