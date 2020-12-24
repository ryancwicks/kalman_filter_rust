use serde::Deserialize;
use std::path::Path;
use std::error;

use plotters::prelude::*;

// Change the alias to `Box<error::Error>`.
type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

const TIME_STEP: f64 = 0.1;
const PROCESS_NOISE_VARIANCE_PRESSURE: f64 = 0.0;//0.0001;
const PROCESS_NOISE_VARIANCE_TEMPERATURE: f64 = 0.0; //0.0000001;

#[derive(Deserialize, Debug, Copy, Clone)]
struct BMP280Measurement {
    pressure: f64,
    p_std: f64,
    temp: f64,
    t_std: f64,
    time: Option<f64>,
}

impl BMP280Measurement {

    fn new(pressure: f64, p_std: f64, temp: f64, t_std: f64, time: f64) -> BMP280Measurement {
        BMP280Measurement {pressure: pressure, p_std: p_std,
                           temp: temp, t_std: t_std,
                           time: Some(time)}
    }
    
    fn load<P: AsRef<Path>>(filename: P) -> Result<Vec<BMP280Measurement>> {

        let mut output_vec = vec![];
        let mut time_count = 0.0f64;

        let mut reader = csv::ReaderBuilder::new().comment(Some(b'#')).from_path(filename)?;
        for record in reader.deserialize() {
            let mut record: BMP280Measurement = record?;
            record.time = Some(time_count.clone());
            time_count += 0.1;
            output_vec.push(record);
        }

        Ok(output_vec)
    }

    fn increment_time (&mut self) {
        self.time = Some(self.time.unwrap() + TIME_STEP);
    }
}

fn main() -> Result<()> {
    let filename = Path::new("BMP280_input.txt");
    
    let data = BMP280Measurement::load(filename).expect("Could not parse the file for the BMP280Measurement.");

    //Initialization
    let pressure_guess = 101000f64;
    let pressure_quess_uncertainty = 1000f64.powf(2.0);
    let temp_quess = 20f64;
    let temp_quess_uncertainy = 5f64.powf(2.0);

    let mut state_estimate = vec![];
    let mut kalman_gain = vec![];

    let mut state_prediction = BMP280Measurement::new(pressure_guess, pressure_quess_uncertainty, temp_quess, temp_quess_uncertainy, 0.0f64); 
    state_estimate.push(state_prediction);
    state_prediction = predict_state(&state_prediction);

    for z in data.iter() {
        let current_gain = update_kalman_gain(&state_prediction, z.p_std, z.t_std);
        let current_estimate = update_state (&state_prediction, &z, current_gain.0, current_gain.1);

        state_prediction = predict_state (&current_estimate);

        kalman_gain.push(current_gain);
        state_estimate.push(current_estimate);

    }


    let minp = -2.0*0.9f64; //10000.0f64;
    let maxp = 2.0*0.9f64; //-10000.0f64;
    let mint = -0.03f64; //10000.0f64;
    let maxt = 0.03f64;////-10000.0f64;

    /*for z in data.iter() {
        if z.pressure > maxp {
            maxp = z.pressure;
        }
        if z.temp > maxt {
            maxt = z.temp;
        }
        if z.pressure < minp {
            minp = z.pressure;
        }
        if z.temp < mint {
            mint = z.temp;
        }
    }

    for z in state_estimate.iter() {
        if z.pressure > maxp {
            maxp = z.pressure;
        }
        if z.temp > maxt {
            maxt = z.temp;
        }
        if z.pressure < minp {
            minp = z.pressure;
        }
        if z.temp < mint {
            mint = z.temp;
        }
    }*/

    // plot the raw data
    let root = BitMapBackend::new("BMP280/input.png", (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption("BMP280 Simulated Kalman Smoother", ("sans-serif", 30).into_font())
        .margin(5)
        .x_label_area_size(35)
        .y_label_area_size(40)
        .right_y_label_area_size(50)
        .build_cartesian_2d(0f32..100f32, minp as f32.. maxp as f32)?
        .set_secondary_coord(0f32..100f32, mint as f32.. maxt as f32);

    chart
        .configure_mesh()
        .y_desc("Pressure (Pa)")
        .x_desc("Time (s)")
        .draw()?;

    chart
        .configure_secondary_axes()
        .y_desc("Temperature (C)")
        .draw()?;

    chart
        .draw_series(LineSeries::new(
            data.iter().map(|x| (x.time.unwrap() as f32, (x.pressure - 101_325f64) as f32)),
            &RGBColor(255, 0, 0).mix(0.2),
        ))?
        .label("Measured Pressure Deviation")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED.mix(0.2)));
        
    chart
        .draw_series(LineSeries::new(
            data.iter().map(|x| (x.time.unwrap() as f32, (0.9) as f32)),
            &RGBColor(255, 0, 0).mix(0.2),
        ))?
        .label("Measured Pressure Error")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED.mix(0.2)));

    chart
        .draw_series(LineSeries::new(
            data.iter().map(|x| (x.time.unwrap() as f32, -(0.9) as f32)),
            &RGBColor(255, 0, 0).mix(0.2),
        ))?;

    chart
        .draw_series(LineSeries::new(
            state_estimate.iter().map(|x| (x.time.unwrap() as f32, (x.pressure - 101_325f64) as f32)),
            &RGBColor(200, 50, 0),
        ))?
        .label("Estimated Pressure Deviation")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RGBColor(200, 50, 0)));

    chart
        .draw_series(LineSeries::new(
            state_estimate.iter().map(|x| (x.time.unwrap() as f32, x.p_std.sqrt() as f32)),
            &RGBColor(150, 100, 0),
        ))?
        .label("Estimated Pressure Std Deviation")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RGBColor(150, 100, 0)));

    chart
        .draw_series(LineSeries::new(
            state_estimate.iter().map(|x| (x.time.unwrap() as f32, -x.p_std.sqrt() as f32)),
            &RGBColor(150, 100, 0),
        ))?;

    chart
        .draw_secondary_series(LineSeries::new(
            data.iter().map(|x| (x.time.unwrap() as f32, (x.temp - 23.2f64) as f32)),
            &RGBColor(0, 0, 255).mix(0.2),
        ))?
        .label("Measured Temperature Deviation")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE.mix(0.2)));

    chart
        .draw_secondary_series(LineSeries::new(
            data.iter().map(|x| (x.time.unwrap() as f32, 0.003 as f32)),
            &RGBColor(0, 0, 255).mix(0.2),
        ))?
        .label("Measured Temperature Error")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE.mix(0.2)));

    chart
        .draw_secondary_series(LineSeries::new(
            data.iter().map(|x| (x.time.unwrap() as f32, 0.003 as f32)),
            &RGBColor(0, 0, 255).mix(0.2),
        ))?;

    chart
        .draw_secondary_series(LineSeries::new(
            state_estimate.iter().map(|x| (x.time.unwrap() as f32, (x.temp - 23.2f64) as f32)),
            &RGBColor(0, 50, 200),
        ))?
        .label("Estimated Temperature Deviation")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RGBColor(0, 50, 200)));


    chart
        .draw_secondary_series(LineSeries::new(
            state_estimate.iter().map(|x| (x.time.unwrap() as f32, x.t_std.sqrt() as f32)),
            &RGBColor(0, 100, 150),
        ))?
        .label("Estimated Temperature Std Deviation")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RGBColor(0, 100, 150)));

    chart
        .draw_secondary_series(LineSeries::new(
            state_estimate.iter().map(|x| (x.time.unwrap() as f32, -x.t_std.sqrt() as f32)),
            &RGBColor(0, 100, 150),
        ))?;

    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::UpperRight)
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    Ok(())
}

/// Update the prediction and covariance and return the next state.
fn predict_state(current_state: &BMP280Measurement) -> BMP280Measurement {
    let mut new_val = current_state.clone();
    new_val.increment_time();
    new_val.p_std += PROCESS_NOISE_VARIANCE_PRESSURE;
    new_val.t_std += PROCESS_NOISE_VARIANCE_TEMPERATURE;

    new_val
}

fn update_kalman_gain(current_estimate: &BMP280Measurement, pressure_error: f64, temp_error: f64) -> (f64, f64) {
    (current_estimate.p_std / (current_estimate.p_std + pressure_error), current_estimate.t_std / ( current_estimate.t_std + temp_error))
}

fn update_state (current_estimate: &BMP280Measurement, current_measure: &BMP280Measurement, pressure_gain: f64, temp_gain: f64) -> BMP280Measurement {
    let pressure = current_estimate.pressure + pressure_gain * (current_measure.pressure - current_estimate.pressure);
    let temp = current_estimate.temp + temp_gain * (current_measure.temp - current_estimate.temp);
    let p_std = (1.0f64 - pressure_gain) * current_estimate.p_std;
    let t_std = (1.0f64 - temp_gain) * current_estimate.t_std;
    let time = current_estimate.time.unwrap();

    BMP280Measurement::new (pressure, p_std, temp, t_std, time)
}