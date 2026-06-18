use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{database::{account::User, assets::{asset::Asset, AssetType}, tags::Tag}, extractor::auth::AuthUser, route::{error::{AppError::{self, BadRequest}, ErrorResponse}}, state::AppState};

#[derive(Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum SortBy {
    MostLiked,
    MostDownloaded,
    DisplayNameSimilarity,
    RecentlyCreated
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchAssetsRequest {
    owner_name: Option<String>,
    asset_type: AssetType,
    display_name: Option<String>,
    #[schema(example = "[\"road\", \"turn\"] (must have all)")]
    tags: Option<Vec<String>>,
    #[schema(example = "1 (returns 1st page)")]
    page: Option<i32>,
    #[schema(minimum = 10, maximum = 25)]
    page_size: Option<i32>,
    sort: Option<SortBy>
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchAssetsResponse {
    total_results: i32,
    page: i32,
    entries_on_page: i32,
    offset: i32,
    total_pages: i32,
    results: Vec<SearchAssetsAsset>
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SearchAssetsAsset {
    asset_name: String,
    author_name: String,
    asset_type: AssetType,
    description: Option<String>,
    tags: Vec<String>,
    total_likes: i64,
    downloads: i64
}

#[utoipa::path(
    post,
    operation_id = "searchAssets",
    path = "/assets/search",
    params (
        (
            "Authorization" = String,
            Header,
            description = "Bearer token for user authentication"
        )
    ),
    request_body = SearchAssetsRequest,
    responses(
        (status = 200, description = "Assets retrieved successfully", body = SearchAssetsResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "asset-fetching"
)]
pub async fn search_assets(
    _: AuthUser,
    State(state): State<AppState>,
    Json(filter): Json<SearchAssetsRequest>
) -> Result<Json<SearchAssetsResponse>, AppError> {
    let mut results: Option<Vec<Asset>> = None;
    let pool = &state.db_pool.clone();

    let sort = filter.sort.unwrap_or(SortBy::RecentlyCreated);
    let mut page_size = 10;
    let mut page_number = 1;

    if let Some(p) = filter.page_size {
        if p > 25 || p < 10 {
            return Err(AppError::BadRequest("Page size must be between 10 and 25 inclusive.".into()));
        }
        page_size = p;
    }

    if let Some(p) = filter.page {
        page_number = p;
    }


    // Filter by most specific thing first and work outwards
    if let Some(owner_name) = filter.owner_name {
        let user = User::get_by_username(pool, &owner_name).await?
            .ok_or(AppError::BadRequest(format!("No user found with username {}", owner_name)))?;
        results = Some(Asset::get_owned_by_id(pool, user.id).await?);
    }

    if let Some(display_name) = filter.display_name {
        if let Some(ref mut r) = results {
            *r = r.iter().cloned()
                .filter(|x| x.display_name.to_ascii_lowercase().contains(&display_name.to_ascii_lowercase()))
                .collect();
        } else {
            results = Some(Asset::filter_by_display_name(pool, &display_name).await?);
        }
        if let Some(ref mut r) = results
            && sort == SortBy::DisplayNameSimilarity {
            *r = sort_by_similarity(r, &display_name);
        }
    } else if sort == SortBy::DisplayNameSimilarity {
        return Err(AppError::BadRequest("Cannot sort by display name similarity if not filtering by display name.".into()));
    }

    if let Some(tags) = filter.tags {
        if let Some(ref mut r) = results {
            let mut filtered = vec![];
            for asset in r.iter() {
                let asset_tags = Tag::get_tags_for_asset_id(pool, asset.id).await?;
                if tags.iter().all(|t| asset_tags.iter().any(|at| at.name == *t)) {
                    filtered.push(asset.clone());
                }
            }
            *r = filtered;
        } else {
            let tag_slices: Vec<&str> = tags.iter().map(|s| s.as_str()).collect();
            let assets = Asset::get_all_with_tag_names(pool, &tag_slices).await?;
            results = Some(assets);
        }
    }

    let results = results.unwrap_or(vec![]);
    let mut results = results.into_iter().filter(|x| x.asset_type == filter.asset_type).collect::<Vec<_>>();
    if results.is_empty() {
        let response = SearchAssetsResponse {
            total_results: 0,
            page: 1,
            entries_on_page: 0,
            offset: 0,
            total_pages: 1,
            results: vec![]
        };

        return Ok(Json(response));
    }

    // we already sorted by display name similarity if that was requested
    if sort == SortBy::MostLiked {
        results.sort_by(|x, y| y.total_likes.cmp(&x.total_likes));
    } else if sort == SortBy::MostDownloaded {
        results.sort_by(|x, y| y.downloads.cmp(&x.downloads));
    } else if sort == SortBy::RecentlyCreated {
        results.sort_by(|x, y| y.created_at.cmp(&x.created_at));
    }

    let total_pages = (results.len() + page_size as usize + 1) / page_size as usize;
    let total_results = results.len();
    let mut chunks = results.chunks(page_size as usize);
    let page = chunks.nth(page_number as usize - 1);
    if page.is_none() {
        return Err(BadRequest("Requested page is beyond the end of all results".into()));
    }
    let page = page.unwrap();
    let offset = page_size * page_number;
    let entries_on_page = page.len();

    let mut out_assets = vec![];
    for entry in page {
        let tags = Tag::get_tags_for_asset_id(pool, entry.id).await?;
        out_assets.push(
            SearchAssetsAsset {
                asset_name: entry.asset_name.clone(),
                author_name: entry.author_name.clone(),
                asset_type: entry.asset_type,
                description: entry.description.clone(),
                tags: tags.iter().map(|x| x.name.clone()).collect::<Vec<_>>(),
                total_likes: entry.total_likes,
                downloads: entry.downloads
            }
        )
    }

    let out = SearchAssetsResponse {
        total_pages: total_pages as i32,
        page: page_number,
        total_results: total_results as i32,
        entries_on_page: entries_on_page as i32,
        offset: offset as i32,
        results: out_assets
    };

    Ok(Json(out))
}

fn sort_by_similarity(assets: &[Asset], query: &str) -> Vec<Asset> {
    let query_lower = query.to_ascii_lowercase();
    let mut sorted = assets.to_vec();
    sorted.sort_by_key(|a| {
        let name = a.display_name.to_ascii_lowercase();
        let tier = if name == query_lower {
            0
        } else if name.starts_with(&query_lower) {
            1
        } else {
            2
        };
        (tier, a.display_name.len())
    });
    sorted
}