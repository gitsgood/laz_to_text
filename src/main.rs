use laz_to_text::{LazInfo, time_it};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut laz_info = LazInfo::default();

    time_it("Reading and organising the data", || {
        laz_info = LazInfo::new().expect("Couldn't read the laz");
    });

    println!("What file extension do you wish your data to have?");
    println!("1: json\nElse: txt");

    let mut users_input = String::new();
    std::io::stdin().read_line(&mut users_input)?;

    let users_will: u8 = match users_input.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            println!("You selected .txt");
            time_it("Printing the txt", || {
                laz_info.print_to_text().expect("Couldn't print the txt");
            });
            return Ok(())
        }
    };

    match users_will {
        1 => {
            println!("You selected .json");
            time_it("Printing the json", || {
                laz_info.print_as_json().expect("Couldn't print the json");
            });
        }
        _ => {
            println!("You selected .txt");
            time_it("Printing the txt", || {
                laz_info.print_to_text().expect("Couldn't print the txt");
            });
        }
    }

    Ok(())
}
