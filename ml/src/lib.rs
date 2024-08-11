use candle_core::{DType, Device, Tensor};
use candle_nn::{linear, loss::mse, AdamW, Module, Optimizer, ParamsAdamW, VarBuilder, VarMap};
use std::error::Error;

#[no_mangle]
pub extern "C" fn _start() {
    let _result = run();
}

#[no_mangle]
pub extern "C" fn run() {
    match greet_internal() {
        Ok(predictions) => {
            println!("Model predictions:");
            for (i, pred) in predictions.iter().enumerate() {
                println!("Prediction {}: {:.4}", i + 1, pred);
            }
        }
        Err(e) => println!("Error: {}", e),
    }
}

fn greet_internal() -> Result<Vec<f32>, Box<dyn Error>> {
    // longitude,latitude,housing_median_age,total_rooms,total_bedrooms,population,households,median_income,median_house_value,ocean_proximity
    let csv_data = r#"-122.27,37.85,40.0,751.0,184.0,409.0,166.0,1.3578,147500.0,NEAR BAY
-118.3,33.95,50.0,1843.0,326.0,892.0,314.0,3.1346,120000.0,<1H OCEAN
-117.65,34.11,29.0,2927.0,634.0,1710.0,623.0,3.6219,176600.0,INLAND
-122.46,37.75,26.0,2192.0,438.0,954.0,456.0,4.5352,374200.0,NEAR BAY
-119.24,36.33,9.0,3289.0,621.0,1866.0,631.0,3.1599,95000.0,INLAND"#;

    let mut features = Vec::new();
    let mut target = Vec::new();

    for line in csv_data.lines() {
        let values: Vec<&str> = line.split(',').collect();

        // Assuming the CSV structure: survived,pclass,age,sibsp,parch,fare
        target.push(values[8].parse::<f32>().unwrap_or(f32::NAN));

        features.extend_from_slice(
            &(0..8)
                .map(|col| values[col].parse::<f32>().unwrap_or(f32::NAN))
                .collect::<Vec<_>>(),
        );
    }

    println!("{features:?}");
    println!("{target:?}");

    let num_features = 8;
    let num_samples = features.len() / num_features;

    // Convert data to tensors
    let features_tensor =
        Tensor::from_slice(&features, &[num_samples, num_features], &Device::Cpu)?;
    let target_tensor = Tensor::from_slice(&target, &[num_samples, 1], &Device::Cpu)?;

    // Define the linear regression model
    let varmap = VarMap::new();
    let vb = VarBuilder::from_varmap(&varmap, DType::F32, &Device::Cpu);
    let model = linear(num_features, 1, vb.pp("linear"))?;

    // Set up the optimizer
    let params = ParamsAdamW {
        lr: 0.1,
        ..Default::default()
    };
    let mut optimizer = AdamW::new(vec![], params)?;

    // Training Loop
    let num_epochs = 100;
    for epoch in 0..num_epochs {
        let predictions = model.forward(&features_tensor)?;
        let loss = mse(&predictions, &target_tensor)?;
        optimizer.backward_step(&loss)?;

        if epoch % 10 == 0 {
            println!("Epoch {}: Loss = {:?}", epoch, loss);
        }
    }

    // Step 9: Make and print predictions
    let predictions = model.forward(&features_tensor)?.squeeze(1)?.to_vec1()?;
    println!("Model predictions: {predictions:?}");

    Ok(predictions)
}
