-- Add migration script here
CREATE TABLE public.discord_oauth2 (
    entry_id serial primary key,
    user_id integer NOT NULL REFERENCES public.users(id) ON DELETE CASCADE UNIQUE,
    discord_user_id bigint NOT NULL UNIQUE,
    created_at TIMESTAMP DEFAULT now()
);

ALTER TABLE public.discord_oauth2 OWNER TO postgres;