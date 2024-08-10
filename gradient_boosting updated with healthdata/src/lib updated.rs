use forust_ml::{GradientBooster, Matrix};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct Feature {
    #[serde(rename = "type")]
    feature_type: String,
    geometry: Geometry,
    properties: Properties,
}

#[derive(Debug, Deserialize)]
struct Geometry {
    #[serde(rename = "type")]
    geometry_type: String,
    coordinates: Vec<f64>,
}

#[derive(Debug, Deserialize)]
struct Properties {
    name: String,
    population: u64,
    health_data: HealthData,
}

#[derive(Debug, Deserialize)]
struct HealthData {
    infectious_diseases: InfectiousDiseases,
    vaccination_rates: VaccinationRates,
    health_infrastructure: HealthInfrastructure,
}

#[derive(Debug, Deserialize)]
struct InfectiousDiseases {
    dengue: u32,
    malaria: u32,
}

#[derive(Debug, Deserialize)]
struct VaccinationRates {
    measles: u32,
    polio: u32,
}

#[derive(Debug, Deserialize)]
struct HealthInfrastructure {
    hospitals: u32,
    clinics: u32,
}

#[no_mangle]
pub extern "C" fn _start() {
    let result = run();
}

#[no_mangle]
pub extern "C" fn run() {
    match greet_internal() {
        Ok(predictions) => {
            println!("Model predictions (first 10):");
            for (i, pred) in predictions.iter().take(10).enumerate() {
                println!("Prediction {}: {:.4}", i + 1, pred);
            }
        },
        Err(e) => println!("Error: {}", e),
    }
}

fn greet_internal() -> Result<Vec<f64>, Box<dyn Error>> {
    // Load and parse the JSON file containing health data
    let file = File::open("healthdata.json")?;
    let reader = BufReader::new(file);
    let health_data: Value = serde_json::from_reader(reader)?;
    let features: Vec<Feature> = serde_json::from_value(health_data["features"].clone())?;

    // Prepare data for the GradientBooster model
    let mut data = Vec::new();
    let mut y = Vec::new();

    for feature in features.iter() {
        // Example of using population and health data to create inputs for the model
        // You should replace this with relevant features based on your use case
        y.push(feature.properties.population as f64); // Using population as the target variable

        // Extract features from the JSON data. For example:
        data.extend_from_slice(&[
            feature.geometry.coordinates[0], // longitude
            feature.geometry.coordinates[1], // latitude
            feature.properties.health_data.infectious_diseases.dengue as f64,
            feature.properties.health_data.infectious_diseases.malaria as f64,
            feature.properties.health_data.health_infrastructure.hospitals as f64,
            feature.properties.health_data.health_infrastructure.clinics as f64,
        ]);
    }

    // Assume the number of features per data point is 6
    let matrix = Matrix::new(&data, y.len(), 6);

    let mut model = GradientBooster::default().set_learning_rate(0.3);
    model.fit_unweighted(&matrix, &y, None)?;

    let predictions = model.predict(&matrix, true);
    Ok(predictions)
}

fn main() -> Result<(), Box<dyn Error>> {
    let features = greet_internal()?;
    // Further processing of `features` as needed

    Ok(())
}
