use laz_to_text::LazTextInfo;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let laz_info = LazTextInfo::new()?;

    laz_info.print_to_text()?;

    Ok(())
}
