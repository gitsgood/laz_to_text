use laz_to_text::{LazInfo, time_it, get_laz_file};
use std::{collections::HashSet, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //let mut laz_info = LazInfo::new()?;
    let mut laz_info = LazInfo::default();

    let mut processed_files: HashSet<PathBuf> = HashSet::new();

    let selected_laz_file = get_laz_file().unwrap();
    processed_files.insert(selected_laz_file.clone());
    
    let file_name = selected_laz_file.file_name().unwrap();

    println!("Selected laz filename: {:?}", file_name);

    time_it("Reading and organising the data", || {
        laz_info = LazInfo::new_with_path(selected_laz_file).expect("Couldn't read the laz");
    });

    loop
    {    
        println!("------------------------------");
        println!("Do you wish to combine another laz file into your output?");
        println!("1: Yes\nElse: No");

        let mut users_input = String::new();
        std::io::stdin().read_line(&mut users_input)?;

        let users_will: u8 = match users_input.trim().parse() {
            Ok(num) => num,
            Err(_) => 2
        };

        match users_will {
            1 => {
                println!("You want more!");
                let mut more_laz_info = LazInfo::default();

                let other_selected_laz_file = get_laz_file().unwrap();
                let other_filename = other_selected_laz_file.file_name().unwrap();

                if processed_files.contains(&other_selected_laz_file) {
                    println!("ATTENTION: You have already processed {:?}", other_filename);
                    println!("Do you want to process it again anyways? ...\n1: YES lol\n2: No, I am a reasonable actor");

                    let mut user_reaally_wants_duplicates = String::new();
                    std::io::stdin().read_line(&mut user_reaally_wants_duplicates)?;

                    let users_will: u8 = match  user_reaally_wants_duplicates.trim().parse() {
                        Ok(num) => num,
                        Err(_) => 2
                    };
                    match users_will {
                        1 => (),
                        _ => continue
                    }
                }
                else {
                    println!("Selected laz filename: {:?}", other_filename);
                }

                time_it("Reading and organising more data", || {
                    more_laz_info = LazInfo::new_with_path(other_selected_laz_file).expect("Couldn't read the laz");
                });
                time_it("Merging the data", || {
                    laz_info.merge(&mut more_laz_info).expect("Couldn't merge...");
                });
                /*
                let mut more_laz_info = LazInfo::new()?;
                laz_info.merge(&mut more_laz_info)?;
                */
            }
            _ => {
                println!("Proceeding to printing...");
                break
            }
        }
    }

    loop
    {    
        println!("------------------------------");
        println!("What file extension do you wish your data to have?");
        println!("1: json\nElse: txt");

        let mut users_input = String::new();
        std::io::stdin().read_line(&mut users_input)?;

        let users_will: u8 = match users_input.trim().parse() {
            Ok(num) => num,
            Err(_) => 2
        };

        match users_will {
            1 => {
                println!("You selected .json");
                //laz_info.print_as_json()?;
                time_it("Printing the json", || {
                    laz_info.print_as_json().expect("Couldn't print the json");
                });
            }
            _ => {
                println!("You selected .txt");
                //laz_info.print_to_text()?;
                time_it("Printing the txt", || {
                    laz_info.print_to_text().expect("Couldn't print the txt");
                });
            }
        }

        println!("Do you wish to print the current collection again?");
        println!("1: YES\n2: No");
        let mut users_second_input = String::new();
        std::io::stdin().read_line(&mut users_second_input)?;

        let users_second_will: u8 = match users_second_input.trim().parse() {
            Ok(num) => num,
            Err(_) => 2
        };

        match users_second_will {
            1 => println!("Repeating printing selection..."),
            _ => {
                println!("Shutting down...");
                break
            }
        }
    }

    Ok(())
}
