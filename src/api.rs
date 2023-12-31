mod auth;
mod brand;
mod category;
mod order;
mod shop;
mod township;
mod user;
mod ward;
mod discount;


use actix_web::web;

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
    cfg.service(shop::get_shops);
    cfg.service(shop::get_shop_by_id);
    cfg.service(township::add_township);
    cfg.service(township::get_townships);
    cfg.service(township::get_township_by_id);
    cfg.service(township::update_township);
    cfg.service(township::delete_township);
    cfg.service(ward::add_ward);
    cfg.service(ward::get_wards);
    cfg.service(ward::get_ward_by_id);
    cfg.service(ward::update_ward);
    cfg.service(ward::delete_ward);
    cfg.service(order::add_order);
    cfg.service(order::get_orders);
    cfg.service(discount::add_discount);
    cfg.service(discount::get_discount_by_id);
    cfg.service(discount::get_discounts);
    cfg.service(discount::update_discount);
    cfg.service(discount::delete_discount);



}
