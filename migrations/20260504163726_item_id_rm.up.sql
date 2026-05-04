-- Drop existing foreign key and constraints
ALTER TABLE public.archive_item_tags
    DROP CONSTRAINT archive_item_tags_archive_item_id_fkey;

ALTER TABLE public.archive_item_tags
    DROP CONSTRAINT archive_item_tags_archive_item_id_tag_id_key;

-- Drop time_trials foreign key constraints and indexes
ALTER TABLE public.time_trials
    DROP CONSTRAINT time_trials_car_id_fkey;

ALTER TABLE public.time_trials
    DROP CONSTRAINT time_trials_stage_id_fkey;

DROP INDEX IF EXISTS idx_time_trials_car_and_stage_id;
DROP INDEX IF EXISTS idx_time_trials_car_id;
DROP INDEX IF EXISTS idx_time_trials_stage_id;

-- Add author and name columns to archive_item_tags
ALTER TABLE public.archive_item_tags
    ADD COLUMN item_author text,
    ADD COLUMN item_name text;

-- Populate the new columns from archive_items
UPDATE public.archive_item_tags ait
SET item_author = ai.author, item_name = ai.name
FROM public.archive_items ai
WHERE ait.archive_item_id = ai.archive_item_id;

-- Make the new columns NOT NULL
ALTER TABLE public.archive_item_tags
    ALTER COLUMN item_author SET NOT NULL,
    ALTER COLUMN item_name SET NOT NULL;

-- Add stage_author, stage_name, car_author, car_name columns to time_trials
ALTER TABLE public.time_trials
    ADD COLUMN stage_author text,
    ADD COLUMN stage_name text,
    ADD COLUMN car_author text,
    ADD COLUMN car_name text;

-- Populate stage columns from archive_items via stage_id
UPDATE public.time_trials tt
SET stage_author = ai.author, stage_name = ai.name
FROM public.archive_items ai
WHERE tt.stage_id = ai.archive_item_id;

-- Populate car columns from archive_items via car_id
UPDATE public.time_trials tt
SET car_author = ai.author, car_name = ai.name
FROM public.archive_items ai
WHERE tt.car_id = ai.archive_item_id;

-- Make the new columns NOT NULL
ALTER TABLE public.time_trials
    ALTER COLUMN stage_author SET NOT NULL,
    ALTER COLUMN stage_name SET NOT NULL,
    ALTER COLUMN car_author SET NOT NULL,
    ALTER COLUMN car_name SET NOT NULL;

-- Drop the old primary key from archive_items
ALTER TABLE public.archive_items
    DROP CONSTRAINT archive_items_pkey;

-- Add new composite primary key to archive_items
ALTER TABLE public.archive_items
    ADD CONSTRAINT archive_items_pkey PRIMARY KEY (author, name);

-- Add new foreign key constraint with composite key for archive_item_tags
ALTER TABLE public.archive_item_tags
    ADD CONSTRAINT archive_item_tags_author_name_fkey 
    FOREIGN KEY (item_author, item_name) REFERENCES public.archive_items(author, name) ON DELETE CASCADE;

-- Add new unique constraint for archive_item_tags
ALTER TABLE public.archive_item_tags
    ADD CONSTRAINT archive_item_tags_author_name_tag_id_key UNIQUE (item_author, item_name, tag_id);

-- Add new foreign key constraints for time_trials
ALTER TABLE public.time_trials
    ADD CONSTRAINT time_trials_stage_author_name_fkey 
    FOREIGN KEY (stage_author, stage_name) REFERENCES public.archive_items(author, name) ON DELETE CASCADE;

ALTER TABLE public.time_trials
    ADD CONSTRAINT time_trials_car_author_name_fkey 
    FOREIGN KEY (car_author, car_name) REFERENCES public.archive_items(author, name) ON DELETE CASCADE;

-- Update unique constraint for time_trials (use composite keys instead of UUIDs)
ALTER TABLE public.time_trials
    DROP CONSTRAINT time_trials_user_id_car_id_stage_id_key;

ALTER TABLE public.time_trials
    ADD CONSTRAINT time_trials_user_id_car_author_name_stage_author_name_key 
    UNIQUE (user_id, car_author, car_name, stage_author, stage_name);

-- Recreate indexes with new column names
CREATE INDEX idx_time_trials_stage_author_name ON public.time_trials(stage_author, stage_name);
CREATE INDEX idx_time_trials_car_author_name ON public.time_trials(car_author, car_name);
CREATE INDEX idx_time_trials_car_and_stage_author_name ON public.time_trials(car_author, car_name, stage_author, stage_name);

-- Drop the old archive_item_id columns
ALTER TABLE public.archive_item_tags
    DROP COLUMN archive_item_id;

ALTER TABLE public.archive_items
    DROP COLUMN archive_item_id;

ALTER TABLE public.time_trials
    DROP COLUMN stage_id,
    DROP COLUMN car_id;
