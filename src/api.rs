mod category;
mod brand;
use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(category::add_category);
    cfg.service(category::get_categories);
    cfg.service(category::get_category_by_id);
    cfg.service(category::update_category);
    cfg.service(category::delete_category);
    cfg.service(brand::add_brand);
    cfg.service(brand::get_brands);
    cfg.service(brand::get_brand_by_id);
    cfg.service(brand::update_brand);
    cfg.service(brand::delete_brand);
}
