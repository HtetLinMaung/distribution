mod brand;
mod category;
use actix_web::web;
mod auth;
mod user;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(auth::login);
    cfg.service(auth::hash_password);
    cfg.service(user::add_user);
    cfg.service(user::get_users);
    cfg.service(user::get_user_by_id);
    cfg.service(user::update_user);
    cfg.service(user::delete_user);
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
