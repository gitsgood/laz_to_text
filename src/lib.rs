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
    pub fn from(input: Point) -> LazPoint {
        let parsed_point = LazPoint { x: (input.x), y: (input.y), z: (input.z) };
        parsed_point
    }

    pub fn new(in_x: f64, in_y: f64, in_z: f64) -> LazPoint {
        let new_point: LazPoint = LazPoint { x: (in_x), y: (in_y), z: (in_z) };
        new_point
    }

    pub fn get_highest_numbers_point(&self, other_point: &LazPoint) -> LazPoint {
        let superior_point = LazPoint::new(
            self.x.max(other_point.x), 
            self.y.max(other_point.y), 
            self.z.max(other_point.z));
        superior_point
    }

    pub fn get_lowest_numbers_point(&self, other_point: &LazPoint) -> LazPoint {
        let inferior_point = LazPoint::new(
            self.x.min(other_point.x), 
            self.y.min(other_point.y), 
            self.z.min(other_point.z));
        inferior_point
    }

    pub fn add(&self, other_point: &LazPoint) -> LazPoint {
        let sum_point = LazPoint::new(
            self.x.add(other_point.x), 
            self.y.add(other_point.y), 
            self.z.add(other_point.z));
        sum_point
    }

    pub fn divide_by_number(&self, number: f64) -> LazPoint {
        let quotient_point = LazPoint::new(
            self.x / number, 
            self.y / number, 
            self.z / number);
        quotient_point
    }

    pub fn copy(&self) -> LazPoint {
        let copy_point = LazPoint::new(
            self.x, 
            self.y, 
            self.z);
        copy_point
    }
}

#[derive(Serialize)]
pub struct LazInfo {
    pub point_count: usize,
    pub maximum_dimensions_point: LazPoint,
    pub minimum_dimensions_point: LazPoint,
    pub mean_dimensions_point: LazPoint,
    pub points: Vec<LazPoint>,
    pub scaled_down_points: Vec<LazPoint>
}

impl LazInfo {
    pub fn new() -> Result<LazInfo, Box<dyn std::error::Error>> {
        let mut reader = Reader::from_path(get_laz_file().unwrap())?;
        let pd = reader.read_all()?;

        let mut count: usize = 0;
        let mut point_vec: Vec<LazPoint> = vec![];
        let mut scaled_point_vec : Vec<LazPoint> = vec![];

        let mut highest_point: LazPoint = LazPoint::new(f64::MIN, f64::MIN, f64::MIN);
        let mut lowest_point: LazPoint = LazPoint::new(f64::MAX, f64::MAX, f64::MAX);
        let mut mean_point: LazPoint = LazPoint::new(0.0, 0.0, 0.0);

        for wrapped_point in pd.points() {
            count += 1;
            let point = wrapped_point?;
            let parsed_point = LazPoint::from(point);
            highest_point = highest_point.get_highest_numbers_point(&parsed_point);
            lowest_point = lowest_point.get_lowest_numbers_point(&parsed_point);
            mean_point = mean_point.add(&parsed_point);

            point_vec.push(parsed_point.copy());
            scaled_point_vec.push(parsed_point);
        }

        let range_x = highest_point.x - lowest_point.x;
        let range_y = highest_point.y - lowest_point.y;
        let range_z = highest_point.z - lowest_point.z;

        let max_range = range_x.max(range_y).max(range_z);

        let scale = if max_range == 0.0 {1.0} else {max_range};

        for unscaled_point in &mut scaled_point_vec {
            unscaled_point.x = (unscaled_point.x - lowest_point.x)/scale;
            unscaled_point.y = (unscaled_point.y - lowest_point.y)/scale;
            unscaled_point.z = (unscaled_point.z - lowest_point.z)/scale;
        }

        let count_ref = &count;
        let count_for_division = *count_ref as f64;

        mean_point = mean_point.divide_by_number(count_for_division);

        Ok(LazInfo { 
            point_count: count, 
            maximum_dimensions_point: highest_point,
            minimum_dimensions_point: lowest_point,
            mean_dimensions_point: mean_point,
            points: point_vec,
            scaled_down_points:  scaled_point_vec})
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