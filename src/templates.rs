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
#[template(path = "new_paste.html")]
pub struct PasteCreationTemplate;

#[derive(Template)]
#[template(path = "components/paste_card.html")]
pub struct PasteCard {
    pub content: String,
}

#[derive(Template)]
#[template(path = "paste.html")]
pub struct PasteTemplate {
    pub paste_card: PasteCard,
}

#[derive(Template)]
#[template(path = "error.html")]
pub struct ErrorTemplate {
    pub error: String,
}
