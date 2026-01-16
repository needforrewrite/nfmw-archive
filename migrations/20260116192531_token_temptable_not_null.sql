-- Add migration script here
ALTER TABLE public.discord_oauth2_token_storage ALTER COLUMN session_token SET NOT NULL;
ALTER TABLE public.discord_oauth2_token_storage ALTER COLUMN discord_token SET NOT NULL;