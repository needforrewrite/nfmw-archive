-- Add archive_item_id back with UUID type
ALTER TABLE public.archive_items
    ADD COLUMN archive_item_id uuid NOT NULL DEFAULT gen_random_uuid();

ALTER TABLE public.archive_item_tags
    ADD COLUMN archive_item_id uuid;

-- Populate archive_item_tags.archive_item_id from archive_items
UPDATE public.archive_item_tags ait
SET archive_item_id = ai.archive_item_id
FROM public.archive_items ai
WHERE ait.item_author = ai.author AND ait.item_name = ai.name;

-- Make archive_item_id NOT NULL in archive_item_tags
ALTER TABLE public.archive_item_tags
    ALTER COLUMN archive_item_id SET NOT NULL;

-- Drop the new composite constraints for archive_item_tags
ALTER TABLE public.archive_item_tags
    DROP CONSTRAINT archive_item_tags_author_name_fkey;

ALTER TABLE public.archive_item_tags
    DROP CONSTRAINT archive_item_tags_author_name_tag_id_key;

-- Add stage_id and car_id columns back to time_trials
ALTER TABLE public.time_trials
    ADD COLUMN stage_id uuid,
    ADD COLUMN car_id uuid;

-- Populate stage_id and car_id from archive_items
UPDATE public.time_trials tt
SET stage_id = ai.archive_item_id
FROM public.archive_items ai
WHERE tt.stage_author = ai.author AND tt.stage_name = ai.name;

UPDATE public.time_trials tt
SET car_id = ai.archive_item_id
FROM public.archive_items ai
WHERE tt.car_author = ai.author AND tt.car_name = ai.name;

-- Make stage_id and car_id NOT NULL
ALTER TABLE public.time_trials
    ALTER COLUMN stage_id SET NOT NULL,
    ALTER COLUMN car_id SET NOT NULL;

-- Drop new composite foreign key constraints for time_trials
ALTER TABLE public.time_trials
    DROP CONSTRAINT time_trials_stage_author_name_fkey;

ALTER TABLE public.time_trials
    DROP CONSTRAINT time_trials_car_author_name_fkey;

-- Drop the new unique constraint for time_trials
ALTER TABLE public.time_trials
    DROP CONSTRAINT time_trials_user_id_car_author_name_stage_author_name_key;

-- Restore old unique constraint for time_trials
ALTER TABLE public.time_trials
    ADD CONSTRAINT time_trials_user_id_car_id_stage_id_key 
    UNIQUE (user_id, car_id, stage_id);

-- Restore old primary key on archive_items
ALTER TABLE public.archive_items
    DROP CONSTRAINT archive_items_pkey;

ALTER TABLE public.archive_items
    ADD CONSTRAINT archive_items_pkey PRIMARY KEY (archive_item_id);

-- Restore old constraints on archive_item_tags
ALTER TABLE public.archive_item_tags
    ADD CONSTRAINT archive_item_tags_archive_item_id_tag_id_key UNIQUE (archive_item_id, tag_id);

ALTER TABLE public.archive_item_tags
    ADD CONSTRAINT archive_item_tags_archive_item_id_fkey 
    FOREIGN KEY (archive_item_id) REFERENCES public.archive_items(archive_item_id) ON DELETE CASCADE;

-- Restore old foreign key constraints for time_trials
ALTER TABLE public.time_trials
    ADD CONSTRAINT time_trials_stage_id_fkey 
    FOREIGN KEY (stage_id) REFERENCES public.archive_items(archive_item_id) ON DELETE CASCADE;

ALTER TABLE public.time_trials
    ADD CONSTRAINT time_trials_car_id_fkey 
    FOREIGN KEY (car_id) REFERENCES public.archive_items(archive_item_id) ON DELETE CASCADE;

-- Drop new indexes for time_trials
DROP INDEX IF EXISTS idx_time_trials_car_and_stage_author_name;
DROP INDEX IF EXISTS idx_time_trials_car_author_name;
DROP INDEX IF EXISTS idx_time_trials_stage_author_name;

-- Restore old indexes for time_trials
CREATE INDEX idx_time_trials_stage_id ON public.time_trials(stage_id);
CREATE INDEX idx_time_trials_car_id ON public.time_trials(car_id);
CREATE INDEX idx_time_trials_car_and_stage_id ON public.time_trials(car_id, stage_id);

-- Drop author and name columns from archive_item_tags
ALTER TABLE public.archive_item_tags
    DROP COLUMN item_author,
    DROP COLUMN item_name;

-- Drop composite key columns from time_trials
ALTER TABLE public.time_trials
    DROP COLUMN stage_author,
    DROP COLUMN stage_name,
    DROP COLUMN car_author,
    DROP COLUMN car_name;
