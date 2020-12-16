use serde::Deserialize;
use std::error;
use std::fmt;
use std::ops::Index;
use std::path::Path;
use std::fmt::Write;

// Change the alias to `Box<error::Error>`.
type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, Clone)]
struct MismatchedVec{
    message: String
}

impl error::Error for MismatchedVec {}

impl fmt::Display for MismatchedVec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "mismatch between vector sizes: {}", self.message)
    }
}

#[derive(Deserialize, Debug, Copy, Clone)]
struct WheelOdometry {
    speed: f64
}

#[derive(Deserialize, Debug, Copy, Clone)]
struct Gyro {
    angular_speed: f64
}

#[derive(Deserialize, Debug, Copy, Clone)]
struct TimeSteps {
    time: f64
}

#[derive(Deserialize, Debug, Copy, Clone)]
struct Attitude {
    angle: f64
}

#[derive(Deserialize, Debug, Copy, Clone)]
struct Position {
    x: f64,
    y: f64
}

#[derive(Debug)]
pub struct Measurement {
    time: TimeSteps,
    wheel: WheelOdometry,
    gyro: Gyro
}

impl fmt::Display for Measurement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} s, {} m/s, {} rad/s", self.time.time, self.wheel.speed, self.gyro.angular_speed)
    }
}

pub struct Measurements {
    data: Vec<Measurement>
}

#[derive(Debug)]
pub struct Pose {
    time: TimeSteps,
    position: Position,
    angle: Attitude,
}

impl fmt::Display for Pose {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} s, ({}, {}) m, {} rad/s", self.time.time, self.position.x, self.position.y, self.angle.angle)
    }
}

pub struct Poses {
    data: Vec<Pose>
}

impl Measurements {
    pub fn load <P: AsRef<Path>> (time_file: P, wheel_file: P, gyro_file: P) -> Result<Measurements> {
        let mut times = vec![];
        let mut reader = csv::ReaderBuilder::new().has_headers(false).from_path(time_file)?;
        for record in reader.deserialize() {
            let record: TimeSteps = record?;
            times.push(record);
        }

        let length = times.len();

        let mut wheel_velocity = Vec::with_capacity(length);
        reader = csv::ReaderBuilder::new().has_headers(false).from_path(wheel_file)?;
        for record in reader.deserialize() {
            let record: WheelOdometry = record?;
            wheel_velocity.push(record);
        }
        wheel_velocity.push(WheelOdometry{speed: 0.0});

        let mut gyros = Vec::with_capacity(length);
        reader = csv::ReaderBuilder::new().has_headers(false).from_path(gyro_file)?;
        for record in reader.deserialize() {
            let record: Gyro = record?;
            gyros.push(record);
        }
        gyros.push(Gyro{angular_speed: 0.0});

        if length != wheel_velocity.len() || length != gyros.len() {
            let mut message = String::new();
            write!(&mut message, "Measurement Load Failed, Counts: time={}, wheel velocity={}, gyros={}", length, wheel_velocity.len(), gyros.len())?;
            return Err(MismatchedVec{message: message}.into()) //converts to box
        }

        let mut measurements =  Measurements {
            data: Vec::with_capacity(length)
        };


        for i in 0.. length {
            let measurement = Measurement {
                time: times[i],
                wheel: wheel_velocity[i],
                gyro: gyros[i]
            };

            measurements.data.push(measurement);
        }
        Ok(measurements)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}

impl Index<usize> for Measurements {
    type Output = Measurement;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl Poses {
    pub fn load <P: AsRef<Path>> (time_file: P, pos_file: P, angle_file: P) -> Result<Poses> {
        let mut times = vec![];
        let mut reader = csv::ReaderBuilder::new().has_headers(false).from_path(time_file)?;
        for record in reader.deserialize() {
            let record: TimeSteps = record?;
            times.push(record);
        }

        let length = times.len();
        let mut positions = Vec::with_capacity(length);
        reader = csv::ReaderBuilder::new().has_headers(false).from_path(pos_file)?;
        for record in reader.deserialize() {
            let record: Position = record?;
            positions.push(record);
        }

        let mut angles = Vec::with_capacity(length);
        reader = csv::ReaderBuilder::new().has_headers(false).from_path(angle_file)?;
        for record in reader.deserialize() {
            let record: Attitude = record?;
            angles.push(record);
        }

        if length != positions.len() || length != angles.len() {
            let mut message = String::new();
            write!(&mut message, "Pose Load Failed, Counts: time={}, position={} angles={}", length, positions.len(), angles.len())?;
            return Err(MismatchedVec{message: message}.into()) //converts to box
        }

        let mut poses = Poses {
            data: Vec::with_capacity( length)
        };

        for i in 0..length {
            let pose = Pose {
                time: times[i],
                position: positions[i],
                angle: angles[i]
            };

            poses.data.push(pose);
        }

        Ok(poses)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}

impl Index<usize> for Poses {
    type Output = Pose;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
