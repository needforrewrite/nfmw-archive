-- Add migration script here
ALTER TABLE public.users ADD COLUMN created_at TIMESTAMP;
ALTER TABLE public.users ALTER COLUMN created_at SET DEFAULT now();