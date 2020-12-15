use serde::Deserialize;
use std::error;
use std::fmt;
use std::ops::Index;
use std::path::Path;

// Change the alias to `Box<error::Error>`.
type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, Clone)]
struct MismatchedVec;

impl fmt::Display for MismatchedVec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "mismatch between vector sizes.")
    }
}

#[derive(Deserialize, Debug)]
struct WheelOdometry {
    speed: f64
}

#[derive(Deserialize, Debug)]
struct Gyro {
    angular_speed: f64
}

#[derive(Deserialize, Debug)]
struct TimeSteps {
    time: f64
}

#[derive(Deserialize, Debug)]
struct Attitude {
    angle: f64
}

#[derive(Deserialize, Debug)]
struct Position {
    x: f64,
    y: f64
}

#[derive(Debug)]
struct Measurement {
    time: TimeSteps,
    wheel: WheelOdometry,
    gyro: Gyro
}

impl fmt::Display for Measurement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} s, {} m/s, {} rad/s", self.time.time, self.wheel.speed, self.gyro.angular_speed)
    }
}

struct Measurements {
    data: Vec<Measurement>
}

#[derive(Debug)]
struct Pose {
    time: TimeSteps,
    position: Position,
    angle: Attitude,
}

impl fmt::Display for Pose {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} s, ({}, {}) m, {} rad/s", self.time.time, self.pos.x, self.pos.y, self.gyro.angle)
    }
}

struct Poses {
    data: Vec<Pose>
}

impl Measurements {
    fn load <P: AsRef<Path>> (time_file: P, wheel_file: P, gyro_file: P) -> Result<Measurements> {
        let mut times = vec![];
        let mut reader = csv::Reader::from_path(time_file)?;
        for record in reader.deserialize() {
            let record: TimeSteps = record?;
            times.push(record);
        }

        let length = times.len();

        let mut wheel_velocity = Vec::with_capacity(length);
        reader = csv::Reader::from_path(wheel_file)?;
        for record in reader.deserialize() {
            let record: WheelOdometry = record?;
            wheel_velocity.push(record);
        }

        let mut gyros: Vec<WheelOdometry> = Vec::with_capacity(length);
        reader = csv::Reader::from_path(gyro_file)?;
        for record in reader.deserialize() {
            let record: Gyro = record?;
            gyros.push(record);
        }

        if length != wheel_velocity.len() || length != gyros.len() {
            MismatchedVec.into() //converts to box
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

    fn len(&self) -> usize {
        self.data.len()
    }
}

impl Index<usize> for Measurements {
    type Output = Measurement;

    fn index(&self, index: usize) -> &Self::Output {
        self.data[index]
    }
}

impl Poses {
    fn load <P: AsRef<Path>> (time_file: P, pos_file: P, angle_file: P) -> Result<Poses> {
        let mut times = vec![];
        let mut reader =csv::Reader::from_path(time_file)?;
        for record in reader.deserialize() {
            let record: TimeSteps = record?;
            times.push(record);
        }

        let length = times.len();
        let mut positions = Vec::with_capacity(length);
        reader = csv::Reader::from_path(pos_file)?;
        for record in reader.deserialize() {
            let record: Position = record?;
            positions.push(record);
        }

        let mut angles = Vec::with_capacity(length);
        reader = csv::Reader::from_path(angle_file)?;
        for record in reader.deserialize() {
            let record: Attitude = record?;
            angles.push(record);
        }

        if length != positions.len() || length != angles.len() {
            MismatchedVec.into() //converts to box
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

    fn len(&self) -> usize {
        self.data.len()
    }
}

impl Index<usize> for Poses {
    type Output = Poses;

    fn index(&self, index: usize) -> &Self::Output {
        self.data[index]
    }
}



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
