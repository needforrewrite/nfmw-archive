-- Add migration script here
ALTER TABLE public.discord_oauth2_token_storage ADD discord_user_id bigint NOT NULL UNIQUE;