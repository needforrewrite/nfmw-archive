-- Add migration script here
ALTER TABLE public.user_tokens drop constraint user_tokens_user_id_fkey,
    add constraint user_tokens_user_id_fkey foreign key (user_id) references public.users(id)
    on delete cascade;