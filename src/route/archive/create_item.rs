use std::os::linux::raw;

use axum::{Json, extract::{Multipart, State}, http::{HeaderMap, StatusCode}};
use serde_json::json;
use sqlx::types::{Uuid, uuid::Version};

use crate::{archive::{ArchiveItemType, parse::parse_file}, db::{archive_item::ArchiveItem, token::UserToken}, state::ThreadSafeState};

pub async fn create_archive_item(State(state): State<ThreadSafeState>, headers: HeaderMap, mut multipart: Multipart) -> axum::response::Result<(StatusCode, Json<serde_json::Value>)> {
    let auth = headers.get("authorization").ok_or((StatusCode::UNAUTHORIZED, Json(json!({"status": "no authorization token provided"}))))?;
    let lock = state.lock().await;

    let authenticated_user = UserToken::get_user_by_token(
        &lock.db_pool, 
        auth.to_str().map_err(|e| 
            (StatusCode::UNAUTHORIZED, Json(json!({"status": "invalid authorization token"})))
        )?.to_owned()).await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"status": "internal database error"}))))?
        .ok_or((StatusCode::UNAUTHORIZED, Json(json!({"status": "invalid authorization token"}))))?;

    // Every item must have a type and text data, but stages may also include a soundtrack. The soundtrack is sent as two parts: st_filename and st_data.
    // st_name is required but st_data is optional for stages. This is because a file with st_name may already exist, in which case the stage can use that.
    // In the event of a st_name clash, the soundtrack file must be renamed by the client first. Descriptive names are highly encouraged...
    let mut st_name: Option<String> = None;
    let mut st_data: Option<Vec<u8>> = None;
    let mut r#type: Option<ArchiveItemType> = None;
    let mut data: Option<String> = None;
    let mut id: Option<Uuid> = None;

    while let Some(field) = multipart.next_field().await? {
        let name = field.name().ok_or((StatusCode::BAD_REQUEST, Json(json!({"status": "invalid multipart"}))))?.to_string();
        let mp_data = field.bytes().await?;

        match &name[..] {
            "type" => {
                let raw_type = mp_data[0];
                r#type = Some(match raw_type {
                    0 => ArchiveItemType::Car,
                    1 => ArchiveItemType::Stage,
                    2 => ArchiveItemType::StagePiece,
                    3 => ArchiveItemType::Wheel,
                    _ => {
                        return Ok((StatusCode::BAD_REQUEST, Json(json!({"status": "invalid item type"}))));
                    }
                })
            },
            "data" => {
                data = Some(String::from_utf8(mp_data.to_vec()).map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"status": "invalid item data"}))))?);
            },
            "st_name" => {
                st_name = Some(String::from_utf8(mp_data.to_vec()).map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"status": "invalid soundtrack name"}))))?)
            },
            "st_data" => {
                st_data = Some(mp_data.to_vec());
            },
            "id" => {
                let string = String::from_utf8(mp_data.to_vec()).map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"status": "invalid id"}))))?;
                let rawid = Uuid::parse_str(&string).map_err(|_| (StatusCode::BAD_REQUEST, Json(json!({"status": "invalid id"}))))?;
                if let Some(v) = rawid.get_version() && v != Version::Random {
                    return Ok((StatusCode::BAD_REQUEST, Json(json!({"status": "invalid id version"}))))
                };

                id = Some(rawid);
            }
            _ => {
                return Ok((StatusCode::BAD_REQUEST, Json(json!({"status": format!("invalid multipart entry: {name}")}))))
            }
        }
    }

    if (st_data.is_some() || st_name.is_some()) && let Some(t) = &r#type && *t != ArchiveItemType::Stage {
        return Ok((StatusCode::BAD_REQUEST, Json(json!({"status": "soundtracks can only be uploaded with stages"}))))
    }

    if r#type.is_none() {
        return Ok((StatusCode::BAD_REQUEST, Json(json!({"status": "item must have a type"}))))
    } else if data.is_none() {
        return Ok((StatusCode::BAD_REQUEST, Json(json!({"status": "item must have data"}))))
    } else if id.is_none() {
        return Ok((StatusCode::BAD_REQUEST, Json(json!({"status": "item must have an id"}))))
    }

    let r#type = r#type.unwrap();
    let data = data.unwrap();

    if data.is_empty() {
        return Ok((StatusCode::BAD_REQUEST, Json(json!({"status": "item must have data"}))))
    }

    let parsed = parse_file(data.clone(), r#type == ArchiveItemType::Stage)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(
            json!({"status": format!("error parsing file: {e}")})
        )))?;

    let id = id.unwrap();
    if id.is_nil() {
        return Ok((StatusCode::BAD_REQUEST, Json(json!({"status": "invalid id"}))));
    }

    let path = format!("{}/{}/{}.txt", lock.config.filestore, r#type.to_string(), id.hyphenated().to_string());

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

    // TODO: process item tags - figure out if this user can insert all the tags
    // (and check if all the provided tags even exist)

    Ok((StatusCode::OK, Json(serde_json::json!({"status": "item created"}))))
}