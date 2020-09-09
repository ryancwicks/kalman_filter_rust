use std::env;
use std::fs::File;
use std::path::Path;
use std::error::Error;
use std::ffi::OsString;

type Time = f64;
type Gyro = f64;
type Wheel = f64;

fn load_csv <P: AsRef<Path>> (filename: P) -> Result<(csv::Reader<File>, usize), Box<dyn Error>> {

    let file = File::open(filename)?;
    let mut rdr = csv::ReaderBuilder::new()
                    .has_headers(false)
                    .from_reader(file);
    let mut length: usize = 0;
    let start_pos = rdr.position().clone();
    for _ in rdr.records() {
        length += 1;
    }
    rdr.seek(start_pos)?;
    return Ok((rdr, length))
}

fn get_nth_arg( n: usize) -> Result<OsString, Box<dyn Error>> {
    match env::args_os().nth( n ) {
        None => Err(From::from(format!("Expected {} argument, got none.", n))),
        Some(file_path) => Ok(file_path),
    }
}

fn run() -> Result<(), Box<dyn Error>> {

    let time_path = get_nth_arg(1)?;
    let mut time_records = load_csv (time_path)?;
    let mut time_vec: Vec<Time> = Vec::with_capacity(time_records.1);

    for result in time_records.0.deserialize() {
        time_vec.push(result?);
    }

    let gyro_path = get_nth_arg(2)?;
    let mut gyro_records = load_csv (gyro_path)?;
    let mut gyro_vec: Vec<Gyro> = Vec::with_capacity(gyro_records.1);

    for result in gyro_records.0.deserialize() {
        gyro_vec.push(result?);
    }

    let wheel_path = get_nth_arg(3)?;
    let mut wheel_records = load_csv (wheel_path)?;
    let mut wheel_vec: Vec<Wheel> = Vec::with_capacity(wheel_records.1);
   
    for result in wheel_records.0.deserialize() {
        wheel_vec.push(result?);
    }
    
    println!{"Times: {}\nGyro: {}\nWheel: {}", time_vec.len(), gyro_vec.len(), wheel_vec.len()};

    return Ok(())
}

fn main() {

    if let Err(error) = run() {
        println!("Failed to run program: {:?}", error);
    }

}