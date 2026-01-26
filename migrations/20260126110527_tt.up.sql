-- Add up migration script here

CREATE TABLE public.time_trials (
    id uuid PRIMARY KEY NOT NULL UNIQUE,
    user_id INTEGER NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,
    stage_id uuid NOT NULL REFERENCES public.archive_items(archive_item_id) ON DELETE CASCADE,
    car_id uuid NOT NULL REFERENCES public.archive_items(archive_item_id) ON DELETE CASCADE,
    created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT NOW(),
    tt_version INTEGER NOT NULL,
    total_ticks INTEGER NOT NULL,
    UNIQUE (user_id, car_id, stage_id)
);

CREATE INDEX idx_time_trials_user_id ON public.time_trials(user_id);
CREATE INDEX idx_time_trials_stage_id ON public.time_trials(stage_id);
CREATE INDEX idx_time_trials_car_id ON public.time_trials(car_id);
CREATE INDEX idx_time_trials_car_and_stage_id ON public.time_trials(car_id, stage_id);
CREATE INDEX idx_time_trials_created_at ON public.time_trials(created_at);