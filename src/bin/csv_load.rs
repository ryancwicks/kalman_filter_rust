use std::env;
use std::fs::File;
use std::path::Path;
use std::error::Error;
use std::ffi::OsString;


fn get_nth_arg( n: usize) -> Result<OsString, Box<dyn Error>> {
    match env::args_os().nth( n ) {
        None => Err(From::from(format!("Expected {} argument, got none.", n))),
        Some(file_path) => Ok(file_path),
    }
}

fn run() -> Result<(), Box<dyn Error>> {

    let time_file = "data/time.csv";
    let gyro_file = "data/u_gyro.csv";
    let wheel_file = "data/u_wheel.csv";
    let pos_file = "data/r_zw_t.csv";
    let angle_file = "data/omega_bt.csv";
 

    let measurements = Measurements::load(time_file, wheel_file, gyro_file)?;
    let poses = Poses::load(time_file, pos_file, angle_file)?;

    println!("{}", measurements[0]);
    println!("...");
    println!("{}\n", measurements[measurements.length()-1]);
    println!("{}", poses[0]);
    println!("...");
    println!("{}", poses[poses.length()-1]);

    Ok(())
}

fn main() {

    if let Err(error) = run() {
        println!("Failed to run program: {:?}", error);
    }

}