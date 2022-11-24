use super::PbChem;
use gbdt::decision_tree::{Data, DataVec};
use gbdt::gradient_boost::GBDT;
use spin;
use std::fs;

// make sure file exists for cargo
static INIT: spin::Once<GBDT> = spin::Once::new();
static JSON: &str = include_str!("../../models/gbdt_0.81_p2.0.json");
static JSON_2_2: &str = include_str!("../../models/gbdt_0.81_p2.2.json");

pub fn get_saved_gbdt_model(polymerase: &PbChem) -> &'static GBDT {
    INIT.call_once(|| {
        let json = match polymerase {
            PbChem::Two => {
                log::info!("Using model for 2.0 chemistry");
                JSON
            }
            PbChem::TwoPointTwo => {
                log::info!("Using model for 2.2 chemistry");
                JSON_2_2
            }
        };
        let temp_file_name = "ft.tmp.model.json";
        fs::write(temp_file_name, json).expect("Unable to write file");
        let model = GBDT::from_xgoost_dump(temp_file_name, "binary:logistic")
            .expect("failed to load model");
        fs::remove_file(temp_file_name).expect("Unable to remove temp model file");
        log::info!("XGBoost model loaded");
        model
    })
}

pub fn predict_with_xgb(windows: &[f32], count: usize, polymerase: &PbChem) -> Vec<f32> {
    if count == 0 {
        return vec![];
    }
    let chunk_size = windows.len() / count;
    let mut gbdt_data: DataVec = Vec::new();
    for window in windows.chunks(chunk_size) {
        let d = Data::new_test_data(window.to_vec(), None);
        gbdt_data.push(d);
    }
    let gbdt_model = get_saved_gbdt_model(polymerase);
    gbdt_model.predict(&gbdt_data)
}