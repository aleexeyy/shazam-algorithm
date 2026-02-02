use actix_files::{Files, NamedFile};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::http::Method;
use actix_web::{HttpResponse, web};
use std::path::PathBuf;

pub fn configure(cfg: &mut web::ServiceConfig, dist_dir: PathBuf) {
    let index_path = dist_dir.join("index.html");

    cfg.service(
        Files::new("/", dist_dir)
            .index_file("index.html")
            .default_handler(move |req: ServiceRequest| {
                let index_path = index_path.clone();
                async move {
                    if req.method() != Method::GET && req.method() != Method::HEAD {
                        return Ok(req.into_response(HttpResponse::NotFound().finish()));
                    }

                    let (req, _pl) = req.into_parts();
                    let file = NamedFile::open(index_path)?;
                    let res = file.into_response(&req);
                    Ok(ServiceResponse::new(req, res))
                }
            }),
    );
}
