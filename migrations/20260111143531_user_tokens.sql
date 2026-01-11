-- Add migration script here

CREATE TABLE public.user_tokens (
    token_id serial PRIMARY KEY,
    user_id integer NOT NULL REFERENCES public.users(id) UNIQUE,
    token text NOT NULL
);

ALTER TABLE public.user_tokens OWNER TO postgres;