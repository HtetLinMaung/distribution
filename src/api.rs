use actix_web::web;
mod user;
mod auth;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(auth::login);
    cfg.service(auth::hash_password);
    cfg.service(user::add_user);
    cfg.service(user::get_users);
    cfg.service(user::get_user_by_id);
    cfg.service(user::update_user);
    cfg.service(user::delete_user);
   
}
