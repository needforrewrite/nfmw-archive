use utoipa::OpenApi;

use crate::route::{
    HealthResponse, account::{
        local::{create::CreateLocalAccountResponse, login::LoginLocalAccountResponse},
        oauth2::{
            discord::{create_account::DiscordCreateAccountResponse, init::DiscordInitResponse},
            poll::PollResponse,
        },
        service_key::{
            create::{CreateServiceKeyResponse, ServiceDescriptor},
            validate::{ValidateServiceKeyRequest, ValidateServiceKeyResponse},
        },
    }, archive::{create_asset::CreateAssetResponse, curation::like_asset::LikeAssetResponse, search_assets::SearchAssetsResponse}, error::ErrorResponse
};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::route::root,
        crate::route::account::local::login::login_local_account,
        crate::route::account::local::create::create_local_account,
        crate::route::account::oauth2::discord::init::discord_oauth_start,
        crate::route::account::oauth2::discord::create_account::discord_create_account,
        crate::route::account::oauth2::poll::poll_oauth,
        crate::route::account::service_key::create::create_service_key,
        crate::route::account::service_key::validate::validate_service_key,
        crate::route::archive::create_asset::create_asset,
        crate::route::archive::get_asset::get_asset,
        crate::route::archive::search_assets::search_assets,
        crate::route::archive::curation::like_asset::set_asset_liked
    ),
    components(
        schemas(
            crate::route::account::local::login::LoginLocalAccountRequest,
            crate::route::account::local::create::CreateLocalAccountRequest,
            crate::route::account::oauth2::discord::create_account::CreateAccountBody,
            HealthResponse,
            ErrorResponse,
            LoginLocalAccountResponse,
            CreateLocalAccountResponse,
            DiscordInitResponse,
            DiscordCreateAccountResponse,
            PollResponse,
            CreateAssetResponse,
            SearchAssetsResponse,
            LikeAssetResponse,
            CreateServiceKeyResponse,
            ServiceDescriptor,
            ValidateServiceKeyRequest,
            ValidateServiceKeyResponse
        )
    ),
    info(
        title = "NFMW Archive API",
        version = "0.1.0",
        description = "REST API for the NFMW archive service"
    ),
    tags(
        (name = "health"),
        (name = "local-auth", description = "Local username/password authentication"),
        (name = "discord-oauth", description = "Discord OAuth2 authentication"),
        (name = "oauth-poll", description = "OAuth2 result polling"),
        (name = "service-keys", description = "Minting and redeeming keys scoped to an external service"),
        (name = "asset-management", description = "Uploading, fetching, and management of assets"),
        (name = "asset-fetching", description = "Fetching and searching of assets"),
        (name = "asset-curation", description = "Liking, featuring, and sorting assets by popularity")
    )
)]
pub struct ApiDoc;
