use spin;
use std::fs;
use tch;

// make sure file exists for cargo
static INIT_PT: spin::Once<tch::CModule> = spin::Once::new();
static PT: &[u8] = include_bytes!("../models/m6ANet_PS00075.best.torch.pt");

pub fn get_saved_pytorch_model() -> &'static tch::CModule {
    INIT_PT.call_once(|| {
        let temp_file_name = "ft.tmp.model.json";
        fs::write(temp_file_name, PT).expect("Unable to write file");
        let model = tch::CModule::load(temp_file_name).expect("Unable to load PyTorch model");
        fs::remove_file(temp_file_name).expect("Unable to remove temp model file");
        log::info!("Model from PyTorch loaded");
        model
    })
}

pub fn predict_with_cnn(windows: &[f32], count: usize) -> Vec<f32> {
    let model = get_saved_pytorch_model();
    let ts = tch::Tensor::of_slice(windows);
    let ts = ts.reshape(&[count.try_into().unwrap(), 6, 15]);
    let x = model.forward_ts(&[ts]).unwrap();
    let w: Vec<f32> = x.try_into().unwrap();
    let z: Vec<f32> = w.chunks(2).map(|c| c[0]).collect();
    log::trace!(
        "{:?} {} {} {}",
        z.len(),
        count,
        z.iter().sum::<f32>() / z.len() as f32,
        w.chunks(2).map(|c| c[1]).sum::<f32>() / z.len() as f32
    );
    z
}
