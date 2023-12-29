use actix_web::{HttpResponse, Responder};
use utils::data::debug_struct::DebugBuffer;

pub async fn debug() -> impl Responder {
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("src/data/debug");

    let content = match std::fs::read_to_string(path.clone()) {
        Ok(file) => file,
        Err(_) => return HttpResponse::Ok().json(Vec::<DebugBuffer>::new()),
    };
    log::debug!("{}", content);
    let vec =
        serde_json::from_str::<Vec<DebugBuffer>>(&content).expect("Deserializing debug file error");

    HttpResponse::Ok().json(vec)
}

pub fn init_debug() {
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("src/data/debug");
    _ = std::fs::remove_file(path);
}
