-- Add migration script here
ALTER TABLE public.users ALTER COLUMN phash DROP NOT NULL;
ALTER TABLE public.users ALTER COLUMN psalt DROP NOT NULL;