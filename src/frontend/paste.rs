use axum::{
    extract::Path,
    Extension,
};

use crate::{
    db::{
        pastes::Paste,
        slugs::{
            Slug,
            SlugString,
        },
    },
    frontend::HtmlPageError,
    http::ApiContext,
    templates::{
        PasteCard,
        PasteCreationTemplate,
        PasteTemplate,
    },
};

/// The paste creation page, presents a form to the user to create a new paste.
pub async fn creation() -> PasteCreationTemplate {
    PasteCreationTemplate
}

/// The paste page, retrieves a paste from the database and presents an HTML page with its content.
pub async fn page(
    ctx: Extension<ApiContext>,
    Path(slug_path): Path<String>,
) -> Result<PasteTemplate, HtmlPageError> {
    // First off, check if the given slug is actually valid.
    if !SlugString::is_valid(&slug_path) {
        return Err(HtmlPageError::InvalidPath(slug_path));
    }

    // Attempt to get a paste with the given slug from the database.
    // If the paste doesn't exist, return a 404.
    let slug = sqlx::query_file_as!(Slug, "sql/get_slug_by_slug.sql", slug_path)
        .fetch_optional(&ctx.db)
        .await
        .map_err(|_| HtmlPageError::DatabaseError)?
        .map_or(Err(HtmlPageError::NotFound), Ok)?;

    // Is the slug actually enabled? If not, return a 404.
    if slug.enabled.is_none() {
        return Err(HtmlPageError::NotFound);
    }

    let paste: Paste = sqlx::query_file_as!(Paste, "sql/get_paste_by_id.sql", slug.paste_id)
        .fetch_optional(&ctx.db)
        .await
        .map_err(|_| HtmlPageError::DatabaseError)?
        .map_or(Err(HtmlPageError::NotFound), Ok)?;

    Ok(PasteTemplate {
        paste_card: PasteCard {
            content: paste.content,
        },
    })
}
