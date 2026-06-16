use utoipa::OpenApi;

use crate::route::{
    HealthResponse,
    account::{
        local::{create::CreateLocalAccountResponse, login::LoginLocalAccountResponse},
        oauth2::{
            discord::{create_account::DiscordCreateAccountResponse, init::DiscordInitResponse},
            poll::PollResponse,
        },
    },
    error::ErrorResponse,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::route::root,
        crate::route::account::local::login::login_local_account,
        crate::route::account::local::create::create_local_account,
        crate::route::account::oauth2::discord::init::discord_oauth_start,
        crate::route::account::oauth2::discord::callback::discord_login_callback,
        crate::route::account::oauth2::discord::create_account::discord_create_account,
        crate::route::account::oauth2::poll::poll_oauth,
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
    )
)]
pub struct ApiDoc;
