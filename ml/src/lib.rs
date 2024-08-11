use forust_ml::{GradientBooster, Matrix};
use std::error::Error;

#[no_mangle]
pub extern "C" fn _start() {
    let _result = run();
}

#[no_mangle]
pub extern "C" fn run() {
    match greet_internal() {
        Ok(predictions) => {
            println!("Model predictions (first 10):");
            for (i, pred) in predictions.iter().take(10).enumerate() {
                println!("Prediction {}: {:.4}", i + 1, pred);
            }
        }
        Err(e) => println!("Error: {}", e),
    }
}

fn greet_internal() -> Result<Vec<f64>, Box<dyn Error>> {
    //     let csv_data = r#"survived,pclass,sex,age,sibsp,parch,fare,embarked,class,who,adult_male,deck,embark_town,alive,alone
    // 0,3,male,22.0,1,0,7.25,S,Third,man,True,,Southampton,no,False
    // 1,1,female,38.0,1,0,71.2833,C,First,woman,False,C,Cherbourg,yes,False
    // 1,3,female,26.0,0,0,7.925,S,Third,woman,False,,Southampton,yes,True
    // 1,1,female,35.0,1,0,53.1,S,First,woman,False,C,Southampton,yes,False
    // 0,3,male,35.0,0,0,8.05,S,Third,man,True,,Southampton,no,True"#;

    // longitude,latitude,housing_median_age,total_rooms,total_bedrooms,population,households,median_income,median_house_value,ocean_proximity
    let csv_data = r#"-122.27,37.85,40.0,751.0,184.0,409.0,166.0,1.3578,147500.0,NEAR BAY
-118.3,33.95,50.0,1843.0,326.0,892.0,314.0,3.1346,120000.0,<1H OCEAN
-117.65,34.11,29.0,2927.0,634.0,1710.0,623.0,3.6219,176600.0,INLAND
-122.46,37.75,26.0,2192.0,438.0,954.0,456.0,4.5352,374200.0,NEAR BAY
-119.24,36.33,9.0,3289.0,621.0,1866.0,631.0,3.1599,95000.0,INLAND"#;

    let mut data = Vec::new();
    let mut y = Vec::new();

    for line in csv_data.lines() {
        let values: Vec<&str> = line.split(',').collect();

        // Assuming the CSV structure: survived,pclass,age,sibsp,parch,fare
        y.push(values[8].parse::<f64>().unwrap_or(f64::NAN));

        data.extend_from_slice(
            &(0..8)
                .map(|col| values[col].parse::<f64>().unwrap_or(f64::NAN))
                .collect::<Vec<_>>(),
        );
    }

    println!("{data:?}");
    println!("{y:?}");

    let matrix = Matrix::new(&data, csv_data.lines().count(), 8);

    let mut model = GradientBooster::default().set_learning_rate(0.3);
    model.fit_unweighted(&matrix, &y, None)?;

    let predictions = model.predict(&matrix, true);
    Ok(predictions)
}
