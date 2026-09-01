use std::{fs::File, io::Write, ops::Add, path::PathBuf, vec};
use native_dialog::{DialogBuilder};
use las::{Point, Reader};
use serde::{Serialize};

#[derive(Serialize)]
pub struct LazPoint {
    pub x: f64,
    pub y: f64,
    pub z: f64
}

impl LazPoint {
    pub fn from(input: Point) -> Result<LazPoint, Box<dyn std::error::Error>> {
        let parsed_point = LazPoint { x: (input.x), y: (input.y), z: (input.z) };
        Ok(parsed_point)
    }
}

#[derive(Serialize)]
pub struct LazInfo {
    pub point_count: usize,
    pub points: Vec<LazPoint>
}

impl LazInfo {
    pub fn new() -> Result<LazInfo, Box<dyn std::error::Error>> {
        let mut reader = Reader::from_path(get_laz_file().unwrap())?;
        let pd = reader.read_all()?;

        let mut count: usize = 0;
        let mut point_vec: Vec<LazPoint> = vec![];

        for wrapped_point in pd.points() {
            count += 1;
            let point = wrapped_point?;
            let parsed_point = LazPoint::from(point)?;
            point_vec.push(parsed_point);
        }

        Ok(LazInfo { point_count: count, points: point_vec })
    }

    pub fn print_to_text(&self) -> Result<(), Box<dyn std::error::Error>> {
        let users_path = get_text_destination("Where do you want your LAZ text file placed?").unwrap();
        let output_path = match users_path.into_os_string().into_string() {
            Ok(string) => string.add("/laz_text.txt"),
            Err(os_string) => { 
                return Err(format!("User's output path contained invalid UTF-8: {:?}", os_string).into());
            }
        };

        let mut file = File::create(output_path)?;

        writeln!(file, "{}" ,&self.point_count)?;

        for point in &self.points {
            writeln!(file, "{:.2}\t{:.2}\t{:.2}", point.x, point.y, point.z)?;
        }

        Ok(())
    }

    pub fn print_as_json(&self) -> Result<(), Box<dyn std::error::Error>> {
        let users_path = get_text_destination("Where do you want your LAZ json file placed?").unwrap();
        let output_path = match users_path.into_os_string().into_string() {
            Ok(string) => string.add("/laz_json.json"),
            Err(os_string) => { 
                return Err(format!("User's output path contained invalid UTF-8: {:?}", os_string).into());
            }
        };

        write_json(&self, &output_path)?;

        Ok(())
    }
}

pub fn get_laz_file() -> Option<PathBuf> {
    let path = DialogBuilder::file()
    //.set_location(&std::env::current_dir().unwrap())
    .set_title("Which LAZ file do you want to convert?")
    .set_location("~/")
    .add_filter("LAZ", ["laz"])
    .open_single_file()
    .show()
    .unwrap()?;

    Some(path)
}

pub fn get_text_destination(message: &str) -> Option<PathBuf> {
    let path = DialogBuilder::file()
    .set_title(message)
    .open_single_dir()
    .show()
    .unwrap()?;

    Some(path)
}

pub fn write_json<S: Serialize, P: AsRef<std::path::Path>>(target: &S, path: P) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::create(path.as_ref())?;
    serde_json::to_writer_pretty(file, target)?;
    Ok(())
}