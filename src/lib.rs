use std::{collections::{HashMap, HashSet}, fs::File, io::{BufReader, BufWriter, Write}, ops::Add, path::{Path, PathBuf}, time::Instant, vec};
use native_dialog::{DialogBuilder};
use las::{Point, Reader};
use serde::{Serialize};
use rayon::prelude::*;

pub mod constants;

pub use constants::*;

#[repr(C)] // Ensures C-compatible memory layout
#[derive(Serialize, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LazPoint {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

impl LazPoint {
    pub fn from(input: Point) -> Result<LazPoint, Box<dyn std::error::Error>> {
        let parsed_point = LazPoint { x: (input.x as f32), y: (input.y as f32), z: (input.z as f32) };
        Ok(parsed_point)
    }

    pub fn new(in_x: f32, in_y: f32, in_z: f32) -> LazPoint {
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

    pub fn divide_by_number(&self, number: f32) -> LazPoint {
        let quotient_point = LazPoint::new(
            self.x / number, 
            self.y / number, 
            self.z / number);
        quotient_point
    }

    pub fn multiply_by_number(&self, number: f32) -> LazPoint {
        let product_point = LazPoint::new(
            self.x * number, 
            self.y * number, 
            self.z * number);
        product_point
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
    pub fn new_with_path(path: PathBuf) -> Result<LazInfo, Box<dyn std::error::Error>> {
        let mut reader = Reader::from_path(path)?;
        let pd = reader.read_all()?;

        
        let point_vec:Vec<LazPoint> = pd.points()
            .map(|p| LazPoint::from(p?))
            .collect::<Result<Vec<_>, _>>()?         
            // Here we make sure that the x y z fields match the order vulkan's glm::vec3 struct.
            // A user taking this data wouldn't need to switch it up when rendering it later on.   
            .par_iter()
            .map(|p| LazPoint {
                x : p.x,
                y : p.z,
                z : -p.y
            }).collect::<Vec<LazPoint>>();

        
        let count = point_vec.len();

        // The monstrosity below contributes in reducing the runtime of this program by about 40ms, which is incredibly significant.
        // Figured a presentation of what exactly happens is in order...
        // Rayon is a parallelisation crate (rust library) that allows one to iterate in parallel over lists such as vectors by utilising all the threads you got.
        let stats = point_vec.par_iter()
            // This here first stage is the fold. Rayon will divide the workload into chunks and pass it off to each thread.
            // Each thread will then perform the operation enclosed in the "fold_op" parameter, basically finding the max, min, and total sum of every chunk.
            .fold(
            ||(
                LazPoint::new(f32::MIN, f32::MIN, f32::MIN), // highest
                LazPoint::new(f32::MAX, f32::MAX, f32::MAX), // lowest
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
                    LazPoint::new(f32::MIN, f32::MIN, f32::MIN),
                    LazPoint::new(f32::MAX, f32::MAX, f32::MAX),
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
        let count_for_division = *count_ref as f32;

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
        let merged_mean = self.mean_dimensions_point.multiply_by_number(self.point_count as f32)
            .add(&other_laz.mean_dimensions_point.multiply_by_number(other_laz.point_count as f32)).divide_by_number(count_sum as f32);

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

    pub fn print_as_text<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        writeln!(writer, "{}" ,&self.point_count)?;

        for point in &self.points {
            writeln!(writer, "{:.2}\t{:.2}\t{:.2}", point.x, point.y, point.z)?;
        }

        writer.flush()?;

        Ok(())
    }

    pub fn print_as_json<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        write_json(&self, path)?;

        Ok(())
    }

    pub fn print_as_binary<P :AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        // Cast the slice of Points into a slice of bytes ([u8])
        let bytes: &[u8] = bytemuck::cast_slice(&self.scaled_down_points);

        writer.write_all(bytes)?;

        Ok(())
    }
}

pub fn process_laz_list(list: Vec<LazInfo>) -> Result<LazInfo, Box<dyn std::error::Error>> {
    //let mut final_laz_info = LazInfo::default();
    /*     
    for mut little_laz in list {
        final_laz_info.merge(&mut little_laz)?;
    } 
    */
    let final_laz_info = list.into_par_iter().reduce(
        || LazInfo::default(), 
        |mut laz_a, mut laz_b|{
            laz_a.merge(&mut laz_b).unwrap();
            laz_a
        }
    );

    Ok(final_laz_info)
}

pub fn get_file_list<P :AsRef<Path>>(path: P) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let files_list: Vec<PathBuf> = walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry.path().extension().and_then(|extension| extension.to_str()) == Some("laz")
        })
        .map(|entry| entry.into_path())
        .collect();

    Ok(files_list)
}

fn compute_hash<P :AsRef<Path>>(path: P) -> Result<blake3::Hash, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = blake3::Hasher::new();
    
    let mut buffer = [0; 8192]; 
    loop {
        let bytes_read = std::io::Read::read(&mut reader, &mut buffer)?;
        if bytes_read == 0 { break; }
        hasher.update(&buffer[..bytes_read]);
    }
    
    Ok(hasher.finalize())
}

pub fn get_unique_file_list(file_list: Vec<PathBuf>) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    // Key: file size (u64), Value: list of paths with that size
    let mut size_groups: HashMap<u64, Vec<PathBuf>> = HashMap::new();

    for entry in file_list {
        let metadata = entry.metadata()?;
        let size = metadata.len();
        size_groups.entry(size).or_default().push(entry);
    }

    let mut final_list = Vec::new();
    let mut seen_hashes = HashSet::new();

    for (_size, paths) in size_groups {
        if paths.len() == 1 {
            // Only one file has this size, it must be unique
            final_list.push(paths[0].clone());
        } else {
            // Multiple files have the same size, we hash them to quickly find out what they're made of
            for path in paths {
                let hash = compute_hash(&path)?;
                // insert returns true if the value was not already present
                if seen_hashes.insert(hash) {
                    final_list.push(path);
                } else {
                    println!("Skipping duplicate file: {:?}", path.file_name().unwrap());
                }
            }
        }
    }

    Ok(final_list)
}

pub fn get_laz_files() -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let path = DialogBuilder::file()
    //.set_location(&std::env::current_dir().unwrap())
    .set_title("Which LAZ files do you want to convert?")
    .set_location("~/")
    .add_filter("LAZ", ["laz"])
    .open_multiple_file()
    .show()?;

    Ok(path)
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

pub fn get_folder(message: &str, suffix_to_append: Option<&str>) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let path = DialogBuilder::file()
    .set_title(message)
    .open_single_dir()
    .show()?
    .unwrap();

    if suffix_to_append.is_none(){
        Ok(path)
    }
    else {
        let output_path = match path.into_os_string().into_string() {
            Ok(string) => string.add(suffix_to_append.unwrap()),
            Err(os_string) => { 
                return Err(format!("User's output path contained invalid UTF-8: {:?}", os_string).into());
            }
        };
        Ok(PathBuf::from(output_path))
    }
}

pub fn write_json<S: Serialize, P: AsRef<Path>>(target: &S, path: P) -> Result<(), Box<dyn std::error::Error>> {
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

pub fn user_querry(querry_msg: &str) -> Result<u8, Box<dyn std::error::Error>> {
    println!("{}", querry_msg);

    let mut users_input = String::new();
    std::io::stdin().read_line(&mut users_input)?;

    let users_will: u8 = match users_input.trim().parse() {
        Ok(num) => num,
        Err(_) => u8::MAX
    };
    Ok(users_will)
}

pub fn user_interface() -> Result<(), Box<dyn std::error::Error>> {
    let mut lazes_info : Vec<LazInfo> = vec![];
    let mut selected_files_list : Vec<PathBuf> = vec![];
    let mut unique_files_list : Vec<PathBuf> = vec![];

    loop 
    {
        match user_querry("------------------------------\nDo you wish to process a single LAZ, or multiple ones?\n1: Just one\n2:A few (same folder)\nElse:ALL of them")? {
            1 => {selected_files_list.push(get_laz_file().unwrap());}
            2 => {selected_files_list.append(&mut get_laz_files()?);}
            _ => {selected_files_list.append(&mut get_file_list(get_folder("Select the folder you wish to recursively scour for LAZ files?", None).unwrap())?);}
        }

        match user_querry("Did you get all the files you wanted?\n1: Yes\nElse: No, I want more")? {
            1 => {println!("Proceeding...");break}
            _ => {println!("Repeating selection...")}
        }
    }

    time_it("Filtering out duplicate files...", ||{unique_files_list = get_unique_file_list(selected_files_list).expect("Couldn't filter the duplicates...")});

    //time_it("Processing all the LAZ files...", || {unique_files_list.iter().for_each(|laz_path| lazes_info.push(LazInfo::new_with_path(laz_path.to_owned()).unwrap()))});
    time_it("Processing all the LAZ files...", || {
        let results: Vec<LazInfo> = unique_files_list.par_iter().map(|laz_path| {LazInfo::new_with_path(laz_path.to_owned()).expect("Couldn't read the laz")}).collect();
        lazes_info.extend(results);
    });

    let laz_info = time_it("Merging the LAZ data into a coherent whole...", || { process_laz_list(lazes_info)})?;

    loop 
    {
        match user_querry("------------------------------\nWhat file extension do you wish your data to have?\n1: json\n2: binary\nElse: txt")? {
            1 => {
                println!("You selected json...");
                let output_path = get_folder("Where do you want your LAZ json file placed?", Some(JSON_NAME))?;
                time_it("Printing the json", || {
                    laz_info.print_as_json(output_path).expect("Couldn't print the json");
                });
            }
            2 => {
                println!("You selected binary...");
                let output_path = get_folder("Where do you want your LAZ binary file placed?", Some(BINARY_NAME))?;
                time_it("Printing the binary", || {
                    laz_info.print_as_binary(output_path).expect("Couldn't print the binary");
                });
            }
            _ => {
                println!("You selected txt...");
                let output_path = get_folder("Where do you want your LAZ txt file placed?", Some(TXT_NAME))?;
                time_it("Printing the txt", || {
                    laz_info.print_as_text(output_path).expect("Couldn't print the txt");
                });
            }
        }

        match user_querry("Do you want to print this in another format?\n1: Yes\nElse: No im good")? {
            1 => {println!("Repeating printing selection process...")}
            _ => {println!("Shutting down...");break}
        }
    }

    Ok(())
}