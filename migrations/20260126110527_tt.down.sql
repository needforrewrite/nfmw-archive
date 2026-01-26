-- Add down migration script here

DROP INDEX IF EXISTS idx_time_trials_created_at;
DROP INDEX IF EXISTS idx_time_trials_car_and_stage_id;
DROP INDEX IF EXISTS idx_time_trials_car_id;
DROP INDEX IF EXISTS idx_time_trials_stage_id;
DROP INDEX IF EXISTS idx_time_trials_user_id;
DROP TABLE IF EXISTS public.time_trials;