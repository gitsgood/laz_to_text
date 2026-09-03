use laz_to_text::{LazInfo};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let laz_info = LazInfo::new()?;

    println!("What file extension do you wish your data to have?");
    println!("1: json\nElse: txt");

    let mut users_input = String::new();
    std::io::stdin().read_line(&mut users_input)?;

    let users_will: u8 = match users_input.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            println!("You selected .txt");
            laz_info.print_to_text()?;
            return Ok(())
        }
    };

    match users_will {
        1 => {
            println!("You selected .json");
            laz_info.print_as_json()?;
        }
        _ => {
            println!("You selected .txt");
            laz_info.print_to_text()?;
        }
    }

    Ok(())
}
