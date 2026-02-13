use sqlx::types::{Uuid, time::PrimitiveDateTime};

// The time trial uuid is the same as the stored time trial data filename in the filestore.
// I.e., "tt_data/{time_trial_entry_id}.timetrial"
// In general, time trials that are an old version will be retained, but the client will not load the ghost itself (only the splits and other metadata).
#[derive(sqlx::FromRow, Debug, Clone)]
pub struct TimeTrialEntry {
    pub id: Uuid,
    pub user_id: i32,
    pub car_id: Uuid,
    pub stage_id: Uuid,
    pub tt_version: i32,
    pub total_ticks: i32,
    pub created_at: Option<PrimitiveDateTime>
}
impl TimeTrialEntry {
    /// Inserts a new time trial entry or updates the created_at timestamp if one already exists for the same user, car, and stage.
    /// The time trial ID is returned, which can be used as the filename to store the actual time trial data file.
    pub async fn insert(pool: &sqlx::PgPool, user_id: i32, car_id: Uuid, stage_id: Uuid, tt_version: i32, total_ticks: i32) -> Result<Self, sqlx::Error> {
        // The file is handled separately; this just creates or updates the DB entry.
        // Since a conflict will return the same entry ID, this can be used by the file handler to replace the existing file.
        let res = sqlx::query_as!(
            Self,
            r#"
            INSERT INTO time_trials (user_id, car_id, stage_id, tt_version, total_ticks)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (user_id, car_id, stage_id) DO UPDATE
            SET created_at = NOW()
            RETURNING id, user_id, car_id, stage_id, tt_version, total_ticks, created_at
            "#,
            user_id,
            car_id,
            stage_id,
            tt_version,
            total_ticks
        )
        .fetch_one(pool)
        .await?;

        Ok(res)
    }

    pub async fn update(pool: &sqlx::PgPool, tt_id: Uuid, tt_version: i32, total_ticks: i32) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE time_trials
            SET tt_version = $1, total_ticks = $2, created_at = NOW()
            WHERE id = $3
            "#,
            tt_version,
            total_ticks,
            tt_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn filter(
        pool: &sqlx::PgPool,
        user_id: Option<i32>,
        car_id: Option<Uuid>,
        stage_id: Option<Uuid>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let mut filtered: Option<Vec<Self>> = None;

        if let Some(uid) = user_id {
           filtered = Some(Self::filter_by_user(pool, uid).await?);
        }

        if let Some(cid) = car_id {
            if let Some(ref f) = filtered {
                filtered = Some(f
                    .iter()
                    .cloned()
                    .filter(|i| i.car_id == cid)
                    .collect());
            } else {
                let car_filtered = Self::filter_by_car(pool, cid).await?;
                filtered = Some(car_filtered);
            }
        }

        if let Some(sid) = stage_id {
            if let Some(ref f) = filtered {
                filtered = Some(f
                    .iter()
                    .cloned()
                    .filter(|i| i.stage_id == sid)
                    .collect());
            } else {
                let stage_filtered = Self::filter_by_stage(pool, sid).await?;
                filtered = Some(stage_filtered);
            }
        }

        Ok(filtered.unwrap_or_else(|| vec![]))
    }

    pub async fn filter_by_user(
        pool: &sqlx::PgPool,
        user_id: i32,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let tts = sqlx::query_as!(
            TimeTrialEntry,
            r#"
            SELECT id, user_id, car_id, stage_id, created_at, tt_version, total_ticks
            FROM time_trials
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_all(pool)
        .await?;

        Ok(tts)
    }

    pub async fn filter_by_car(
        pool: &sqlx::PgPool,
        car_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let tts = sqlx::query_as!(
            TimeTrialEntry,
            r#"
            SELECT id, user_id, car_id, stage_id, created_at, tt_version, total_ticks
            FROM time_trials
            WHERE car_id = $1
            "#,
            car_id
        )
        .fetch_all(pool)
        .await?;

        Ok(tts)
    }

    pub async fn filter_by_stage(
        pool: &sqlx::PgPool,
        stage_id: Uuid,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let tts = sqlx::query_as!(
            TimeTrialEntry,
            r#"
            SELECT id, user_id, car_id, stage_id, created_at, tt_version, total_ticks
            FROM time_trials
            WHERE stage_id = $1
            "#,
            stage_id
        )
        .fetch_all(pool)
        .await?;

        Ok(tts)
    }

    pub async fn delete(
        pool: &sqlx::PgPool,
        tt_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            DELETE FROM time_trials
            WHERE id = $1
            "#,
            tt_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}