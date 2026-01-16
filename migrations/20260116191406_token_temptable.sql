-- Add migration script here
CREATE TABLE public.discord_oauth2_token_storage(
    session_token text UNIQUE,
    discord_token text,
    created_at timestamp default now()
);

ALTER TABLE public.discord_oauth2_token_storage OWNER TO postgres;
