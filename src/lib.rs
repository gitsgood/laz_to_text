use std::{fs::File, io::Write, ops::Add, path::PathBuf, vec};
use native_dialog::{DialogBuilder};
use las::{Reader};

pub struct LazTextInfo {
    pub point_count: usize,
    pub points: Vec<las::Point>
}

impl LazTextInfo {
    pub fn new() -> Result<LazTextInfo, Box<dyn std::error::Error>> {
        let mut reader = Reader::from_path(get_laz_file().unwrap())?;
        let pd = reader.read_all()?;

        let mut count: usize = 0;
        let mut point_vec: Vec<las::Point> = vec![];

        for wrapped_point in pd.points() {
            count += 1;
            let point = wrapped_point?;
            point_vec.push(point);
        }

        Ok(LazTextInfo { point_count: count, points: point_vec })
    }

    pub fn print_to_text(&self) -> Result<(), Box<dyn std::error::Error>> {
        let users_path = get_text_destination().unwrap();
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

pub fn get_text_destination() -> Option<PathBuf> {
    let path = DialogBuilder::file()
    .set_title("Where do you want your LAZ text file placed?")
    .open_single_dir()
    .show()
    .unwrap()?;

    Some(path)
}