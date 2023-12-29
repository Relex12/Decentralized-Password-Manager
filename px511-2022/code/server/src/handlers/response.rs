use actix_web::HttpResponse;
use utils::data::comm_struct::EsiPassMsg;

pub fn create_server_resp(request: Vec<EsiPassMsg>) -> HttpResponse {
    HttpResponse::Ok().json(request)
}
