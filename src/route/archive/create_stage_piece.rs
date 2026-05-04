use std::{collections::HashSet};

use axum::{Json, extract::{Multipart, State}, http::{HeaderMap, StatusCode}};
use serde_json::json;
use sqlx::types::{Uuid, uuid::Version};

use crate::{archive::{ArchiveItemType, parse::parse_file}, db::{archive::{archive_item::ArchiveItem, archive_item_tag::ArchiveItemTag, archive_tag::ArchiveTag, archive_tag_ownership::user_can_assign_tag}, token::UserToken}, state::ThreadSafeState};

pub async fn create_stage_piece(State(state): State<ThreadSafeState>, headers: HeaderMap, mut multipart: Multipart) -> axum::response::Result<(StatusCode, Json<serde_json::Value>)> {
    let auth = headers.get("authorization").ok_or((StatusCode::UNAUTHORIZED, Json(json!({"status": "no authorization token provided"}))))?;
    let lock = state.lock().await;

    let authenticated_user = UserToken::get_user_by_token(
        &lock.db_pool, 
        auth.to_str().map_err(|e| 
            (StatusCode::UNAUTHORIZED, Json(json!({"status": "invalid authorization token"})))
        )?).await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"status": "internal database error"}))))?
        .ok_or((StatusCode::UNAUTHORIZED, Json(json!({"status": "invalid authorization token"}))))?;

    let r#type: ArchiveItemType = ArchiveItemType::StagePiece;
    let mut data: Option<String> = None;
    let id: Uuid = Uuid::new_v4();

    while let Some(field) = multipart.next_field().await? {
        let name = field.name().ok_or((StatusCode::BAD_REQUEST, Json(json!({"status": "invalid multipart"}))))?.to_string();
        let mp_data = field.bytes().await?;

        match &name[..] {
            "data" => {
                data = Some(String::from_utf8(mp_data.to_vec()).map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"status": "invalid item data"}))))?);
            },
            _ => {
                return Ok((StatusCode::BAD_REQUEST, Json(json!({"status": format!("invalid multipart entry: {name}")}))))
            }
        }
    }

    let data = data.unwrap_or(Err((StatusCode::BAD_REQUEST, Json(json!({"status": "item must have data"}))))?);
    if data.is_empty() {
        return Ok((StatusCode::BAD_REQUEST, Json(json!({"status": "item must have data"}))))
    }

    let mut parsed = parse_file(data.clone(), false)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(
            json!({"status": format!("error parsing file: {e}")})
        )))?;

    // deduplicate tags
    parsed.tags = parsed.tags.into_iter().collect::<HashSet<_>>().into_iter().collect::<Vec<_>>();

    if parsed.tags.len() > 5 {
        return Ok((StatusCode::BAD_REQUEST, Json(json!({"status": "each item is limited to a maximum of 5 tags"}))))
    }

    if id.is_nil() {
        return Ok((StatusCode::BAD_REQUEST, Json(json!({"status": "invalid id"}))));
    }

    // TODO: for any non-admin user, check that the author is the same as the authenticated username.
    // In the client, the author field should be automatically populated with the correct value.
    if parsed.author.is_none() || parsed.author.as_ref().unwrap().is_empty() {
        return Ok((StatusCode::BAD_REQUEST, Json(json!({"status": "item must have an author"}))));
    }

    let path = format!("{}/{}/{}/{}.txt", lock.config.filestore, r#type.dir_name(), parsed.author.clone().unwrap(), id.hyphenated().to_string());

    let ex = std::fs::exists(&path);
    if let Ok(e) = ex && e == false {
        return Ok((StatusCode::BAD_REQUEST, Json(json!({"status": "id already in use"}))));
    } else if let Err(e) = ex {
        return Ok((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"status": format!("internal filesystem error on id lookup: {e}")}))));
    }

    if let Err(e) = std::fs::write(&path, data.as_bytes()) {
        return Ok((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"status": format!("internal filesystem error on item save: {e}")}))));
    }

    let item = ArchiveItem {
        archive_item_id: id,
        path,
        r#type: r#type.to_string(),
        name: parsed.name,
        created_at: None,
        author: parsed.author,
        owner_user_id: authenticated_user.user_id
    };
    item.insert(&lock.db_pool).await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"status": "internal database error on item save"}))))?;

    for tag in parsed.tags {
        let id = ArchiveTag::get_id_from_name(&lock.db_pool, tag.clone()).await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"status": "internal database error on tag lookup"}))))?
            .ok_or((StatusCode::BAD_REQUEST, Json(json!({"status": format!("provided tag {tag} does not exist")}))))?;

        let user_can_assign = user_can_assign_tag(&lock.db_pool, id, authenticated_user.user_id).await
            .map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({"status": format!("{e}")}))))?;

        if !user_can_assign {
            return Ok((StatusCode::FORBIDDEN, Json(json!({"status": "you don't have permission to assign this tag"}))));
        }

        let relation = ArchiveItemTag {
            archive_item_id: item.archive_item_id,
            tag_id: id
        };

        relation.insert(&lock.db_pool).await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"status": "internal database error on tag relation insertion"}))))?;
    }

    Ok((StatusCode::OK, Json(serde_json::json!({"status": format!("stage piece created", id.hyphenated().to_string()), "id": id.hyphenated().to_string()}))))
}