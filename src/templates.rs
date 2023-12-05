use askama::Template;

use crate::db::users::User;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub user: Option<User>,
}

#[derive(Template)]
#[template(path = "auth.html")]
pub struct AuthTemplate;

#[derive(Template)]
#[template(path = "paste.html")]
pub struct PasteTemplate;
