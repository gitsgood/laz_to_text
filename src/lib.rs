use std::{fs::File, io::Write, io::BufWriter, ops::Add, path::PathBuf, vec, time::Instant};
use native_dialog::{DialogBuilder};
use las::{Point, Reader};
use serde::{Serialize};
use rayon::prelude::*;

#[derive(Serialize, Clone, Copy)]
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

    pub fn multiply_by_number(&self, number: f64) -> LazPoint {
        let product_point = LazPoint::new(
            self.x * number, 
            self.y * number, 
            self.z * number);
        product_point
    }

    /*
    pub fn copy(&self) -> LazPoint {
        let copy_point = LazPoint::new(
            self.x, 
            self.y, 
            self.z);
        copy_point
    }
    */
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
        /*
        let mut current_path = std::env::current_dir()?;
        current_path.push("32-1-517-154-07.laz");
        let mut reader = Reader::from_path(current_path)?;
        */
        let pd = reader.read_all()?;

        
        let point_vec:Vec<LazPoint> = pd.points()
            .map(|p| LazPoint::from(p?))
            .collect::<Result<Vec<_>, _>>()?;
        let count = point_vec.len();

        // The monstrosity below contributes in reducing the runtime of this program by about 40ms, which is incredibly significant.
        // Figured a presentation of what exactly happens is in order...
        // Rayon is a parallelisation crate (rust library) that allows one to iterate in parallel over lists such as vectors by utilising all the threads you got.
        let stats = point_vec.par_iter()
            // This here first stage is the fold. Rayon will divide the workload into chunks and pass it off to each thread.
            // Each thread will then perform the operation enclosed in the "fold_op" parameter, basically finding the max, min, and total sum of every chunk.
            .fold(
            ||(
                LazPoint::new(f64::MIN, f64::MIN, f64::MIN), // highest
                LazPoint::new(f64::MAX, f64::MAX, f64::MAX), // lowest
                LazPoint::new(0.0, 0.0, 0.0),               // sum
            ),
            |mut point_throuple: (LazPoint, LazPoint, LazPoint), point_from_vector: &LazPoint|{
                point_throuple.0 = point_throuple.0.get_highest_numbers_point(point_from_vector);
                point_throuple.1 = point_throuple.1.get_lowest_numbers_point(point_from_vector);
                point_throuple.2 = point_throuple.2.add(point_from_vector);
                point_throuple
            })
            // Once each thread is done, we "reduce" the chunks by performing an operation on whatever it is they returned.
            // In this case, each thread comes back with a "throuple" (tuple containing 3 elements).
            // We then operate on them in order to find the max, min and sum of ALL the threads.
            // For thousands of points, the below would have to only compare a handful of numbers to find the wanted result.
            .reduce(
                || (
                    LazPoint::new(f64::MIN, f64::MIN, f64::MIN),
                    LazPoint::new(f64::MAX, f64::MAX, f64::MAX),
                    LazPoint::new(0.0, 0.0, 0.0),
                ), 
                |mut another_point_throuple: (LazPoint, LazPoint, LazPoint), previous_point_throuple: (LazPoint, LazPoint, LazPoint)| {
                    another_point_throuple.0 = another_point_throuple.0.get_highest_numbers_point(&previous_point_throuple.0);
                    another_point_throuple.1 = another_point_throuple.1.get_lowest_numbers_point(&previous_point_throuple.1);
                    another_point_throuple.2 = another_point_throuple.2.add(&previous_point_throuple.2);
                    another_point_throuple
                },
            );
        let (highest_point, lowest_point, total_sum) = stats;
        

        /*
        let count: usize = pd.points().count();
        let mut point_vec: Vec<LazPoint> = vec![];
        point_vec.reserve(count);
        let mut scaled_point_vec : Vec<LazPoint> = vec![];
        scaled_point_vec.reserve(count);

        let mut highest_point: LazPoint = LazPoint::new(f64::MIN, f64::MIN, f64::MIN);
        let mut lowest_point: LazPoint = LazPoint::new(f64::MAX, f64::MAX, f64::MAX);
        let mut total_sum: LazPoint = LazPoint::new(0.0, 0.0, 0.0);
        

        for wrapped_point in pd.points() {
            let point = wrapped_point?;
            let parsed_point = LazPoint::from(point)?;
            highest_point = highest_point.get_highest_numbers_point(&parsed_point);
            lowest_point = lowest_point.get_lowest_numbers_point(&parsed_point);
            total_sum = total_sum.add(&parsed_point);

            point_vec.push(parsed_point.clone());
            scaled_point_vec.push(parsed_point);
        }
        */

        let range_x = highest_point.x - lowest_point.x;
        let range_y = highest_point.y - lowest_point.y;
        let range_z = highest_point.z - lowest_point.z;

        let max_range = range_x.max(range_y).max(range_z);

        let scale = if max_range == 0.0 {1.0} else {max_range / 100.0};

        let mut scaled_point_vec = point_vec.clone();

        /*
        scaled_point_vec.par_iter_mut().for_each(|unscaled_point| {
            unscaled_point.x = (unscaled_point.x - lowest_point.x) / scale;
            unscaled_point.y = (unscaled_point.y - lowest_point.y) / scale;
            unscaled_point.z = (unscaled_point.z - lowest_point.z) / scale;
        });
        */

        
        for unscaled_point in &mut scaled_point_vec {
            unscaled_point.x = (unscaled_point.x - lowest_point.x)/scale;
            unscaled_point.y = (unscaled_point.y - lowest_point.y)/scale;
            unscaled_point.z = (unscaled_point.z - lowest_point.z)/scale;
        }

        let count_ref = &count;
        let count_for_division = *count_ref as f64;

        let mean_point = total_sum.divide_by_number(count_for_division);

        Ok(LazInfo { 
            point_count: count, 
            maximum_dimensions_point: highest_point,
            minimum_dimensions_point: lowest_point,
            mean_dimensions_point: mean_point,
            points: point_vec,
            scaled_down_points:  scaled_point_vec})
    }

    pub fn new_with_path(path: PathBuf) -> Result<LazInfo, Box<dyn std::error::Error>> {
        let mut reader = Reader::from_path(path)?;
        let pd = reader.read_all()?;

        
        let point_vec:Vec<LazPoint> = pd.points()
            .map(|p| LazPoint::from(p?))
            .collect::<Result<Vec<_>, _>>()?;
        let count = point_vec.len();

        // The monstrosity below contributes in reducing the runtime of this program by about 40ms, which is incredibly significant.
        // Figured a presentation of what exactly happens is in order...
        // Rayon is a parallelisation crate (rust library) that allows one to iterate in parallel over lists such as vectors by utilising all the threads you got.
        let stats = point_vec.par_iter()
            // This here first stage is the fold. Rayon will divide the workload into chunks and pass it off to each thread.
            // Each thread will then perform the operation enclosed in the "fold_op" parameter, basically finding the max, min, and total sum of every chunk.
            .fold(
            ||(
                LazPoint::new(f64::MIN, f64::MIN, f64::MIN), // highest
                LazPoint::new(f64::MAX, f64::MAX, f64::MAX), // lowest
                LazPoint::new(0.0, 0.0, 0.0),               // sum
            ),
            |mut point_throuple: (LazPoint, LazPoint, LazPoint), point_from_vector: &LazPoint|{
                point_throuple.0 = point_throuple.0.get_highest_numbers_point(point_from_vector);
                point_throuple.1 = point_throuple.1.get_lowest_numbers_point(point_from_vector);
                point_throuple.2 = point_throuple.2.add(point_from_vector);
                point_throuple
            })
            // Once each thread is done, we "reduce" the chunks by performing an operation on whatever it is they returned.
            // In this case, each thread comes back with a "throuple" (tuple containing 3 elements).
            // We then operate on them in order to find the max, min and sum of ALL the threads.
            // For thousands of points, the below would have to only compare a handful of numbers to find the wanted result.
            .reduce(
                || (
                    LazPoint::new(f64::MIN, f64::MIN, f64::MIN),
                    LazPoint::new(f64::MAX, f64::MAX, f64::MAX),
                    LazPoint::new(0.0, 0.0, 0.0),
                ), 
                |mut another_point_throuple: (LazPoint, LazPoint, LazPoint), previous_point_throuple: (LazPoint, LazPoint, LazPoint)| {
                    another_point_throuple.0 = another_point_throuple.0.get_highest_numbers_point(&previous_point_throuple.0);
                    another_point_throuple.1 = another_point_throuple.1.get_lowest_numbers_point(&previous_point_throuple.1);
                    another_point_throuple.2 = another_point_throuple.2.add(&previous_point_throuple.2);
                    another_point_throuple
                },
            );
        let (highest_point, lowest_point, total_sum) = stats;
        
        let range_x = highest_point.x - lowest_point.x;
        let range_y = highest_point.y - lowest_point.y;
        let range_z = highest_point.z - lowest_point.z;

        let max_range = range_x.max(range_y).max(range_z);

        let scale = if max_range == 0.0 {1.0} else {max_range / 100.0};

        let mut scaled_point_vec = point_vec.clone();
        
        for unscaled_point in &mut scaled_point_vec {
            unscaled_point.x = (unscaled_point.x - lowest_point.x)/scale;
            unscaled_point.y = (unscaled_point.y - lowest_point.y)/scale;
            unscaled_point.z = (unscaled_point.z - lowest_point.z)/scale;
        }

        let count_ref = &count;
        let count_for_division = *count_ref as f64;

        let mean_point = total_sum.divide_by_number(count_for_division);

        Ok(LazInfo { 
            point_count: count, 
            maximum_dimensions_point: highest_point,
            minimum_dimensions_point: lowest_point,
            mean_dimensions_point: mean_point,
            points: point_vec,
            scaled_down_points:  scaled_point_vec})
    }

    pub fn merge(&mut self, other_laz: &mut LazInfo) -> Result<(), Box<dyn std::error::Error>> {
        let count_sum = self.point_count + other_laz.point_count;
        let merged_highest = self.maximum_dimensions_point.get_highest_numbers_point(&other_laz.maximum_dimensions_point);
        let merged_lowest = self.minimum_dimensions_point.get_lowest_numbers_point(&other_laz.minimum_dimensions_point);
        let merged_mean = self.mean_dimensions_point.multiply_by_number(self.point_count as f64)
            .add(&other_laz.mean_dimensions_point.multiply_by_number(other_laz.point_count as f64)).divide_by_number(count_sum as f64);

        self.point_count = count_sum;
        self.maximum_dimensions_point = merged_highest;
        self.minimum_dimensions_point = merged_lowest;
        self.mean_dimensions_point = merged_mean;
        self.points.append(&mut other_laz.points);
        //self.scaled_down_points.clear();
        //self.scaled_down_points.reserve(count_sum);

        // Recalculating new scaling for the points...
        let range_x = self.maximum_dimensions_point.x - self.minimum_dimensions_point.x;
        let range_y = self.maximum_dimensions_point.y - self.minimum_dimensions_point.y;
        let range_z = self.maximum_dimensions_point.z - self.minimum_dimensions_point.z;

        let max_range = range_x.max(range_y).max(range_z);

        let scale = if max_range == 0.0 {1.0} else {max_range / 100.0};

        let mut scaled_point_vec = self.points.clone();

        /*
        for unscaled_point in &mut scaled_point_vec {
            unscaled_point.x = (unscaled_point.x - self.minimum_dimensions_point.x)/scale;
            unscaled_point.y = (unscaled_point.y - self.minimum_dimensions_point.y)/scale;
            unscaled_point.z = (unscaled_point.z - self.minimum_dimensions_point.z)/scale;
        }
        */
        
        // Parallelising this seems to actually increase the speed of the first merge by a few ms...
        scaled_point_vec.par_iter_mut().for_each(|unscaled_point| {
            unscaled_point.x = (unscaled_point.x - self.minimum_dimensions_point.x) / scale;
            unscaled_point.y = (unscaled_point.y - self.minimum_dimensions_point.y) / scale;
            unscaled_point.z = (unscaled_point.z - self.minimum_dimensions_point.z) / scale;
        });

        self.scaled_down_points = scaled_point_vec;


        Ok(())
    }

    pub fn default() -> LazInfo {
        let default = LazInfo{
            point_count: 0,
            maximum_dimensions_point: LazPoint { x: 0.0, y: 0.0, z: 0.0 },
            minimum_dimensions_point: LazPoint { x: 0.0, y: 0.0, z: 0.0 },
            mean_dimensions_point: LazPoint { x: 0.0, y: 0.0, z: 0.0 },
            points: vec![],
            scaled_down_points: vec![]
        };
        default
    }

    pub fn print_to_text(&self) -> Result<(), Box<dyn std::error::Error>> {
        let users_path = get_text_destination("Where do you want your LAZ text file placed?").unwrap();
        let output_path = match users_path.into_os_string().into_string() {
            Ok(string) => string.add("/laz_text.txt"),
            Err(os_string) => { 
                return Err(format!("User's output path contained invalid UTF-8: {:?}", os_string).into());
            }
        };
        /*
        let mut output_path = std::env::current_dir()?;
        output_path.push("laz_text.txt");
        */

        let file = File::create(output_path)?;
        let mut writer = BufWriter::new(file);

        writeln!(writer, "{}" ,&self.point_count)?;

        for point in &self.points {
            writeln!(writer, "{:.2}\t{:.2}\t{:.2}", point.x, point.y, point.z)?;
        }

        writer.flush()?;

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
        /* 
        let mut output_path = std::env::current_dir()?;
        output_path.push("laz_text.json");
        */

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

    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, target)?;

    //serde_json::to_writer_pretty(file, target)?;
    Ok(())
}

pub fn time_it<F, R>(name: &str, f: F) -> R 
where 
    F: FnOnce() -> R 
{
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();
    println!("{}: took {:?}", name, duration);
    result
}