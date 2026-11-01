pub mod limit_exceeded;
pub mod on_create_record;

use std::sync::atomic::{AtomicBool, Ordering};
use deadpool_redis::{redis, Connection};
use redis::AsyncCommands;
use thiserror::Error;

static DISABLED: AtomicBool = AtomicBool::new(false);

#[derive(Error, Debug)]
pub enum RateLimiterError {
    #[error("Limit exceeded: wait {seconds_to_wait} seconds")]
    LimitExceeded {
        seconds_to_wait: isize,
        limit_type: String,
        error_code: Option<String>,
    },
    #[error(transparent)]
    RedisError(#[from] redis::RedisError),
    #[error(transparent)]
    PoolError(#[from] deadpool_redis::PoolError),
}

pub struct RateLimiter {
    pub max: isize,
    pub secs: isize,
    pub user_id: Option<i32>,
    pub is_staff: bool,
    pub limit_type: String,
    pub key: String,
    pub global: bool,
    pub aggressive: bool,
    pub error_code: Option<String>,
    pub apply_limit_to_staff: bool,
    pub staff_limit: Option<(isize, isize)>,
}

const PERFORM_LUA: &str = r#"
      local now = tonumber(ARGV[1])
      local secs = tonumber(ARGV[2])
      local max = tonumber(ARGV[3])

      local key = KEYS[1]


      if ((tonumber(redis.call("LLEN", key)) < max) or
          (now - tonumber(redis.call("LRANGE", key, -1, -1)[1])) >= secs) then
        redis.call("LPUSH", key, now)
        redis.call("LTRIM", key, 0, max - 1)
        redis.call("EXPIRE", key, secs * 2)

        return 1
      else
        return 0
      end
"#;

const PERFORM_LUA_AGGRESSIVE: &str = r#"
      local now = tonumber(ARGV[1])
      local secs = tonumber(ARGV[2])
      local max = tonumber(ARGV[3])

      local key = KEYS[1]

      local return_val = 0

      if ((tonumber(redis.call("LLEN", key)) < max) or
          (now - tonumber(redis.call("LRANGE", key, -1, -1)[1])) >= secs) then
        return_val = 1
      else
        return_val = 0
      end

      redis.call("LPUSH", key, now)
      redis.call("LTRIM", key, 0, max - 1)
      redis.call("EXPIRE", key, secs * 2)

      return return_val
"#;

impl RateLimiter {
    pub fn key_prefix() -> &'static str {
        "l-rate-limit3:"
    }

    pub fn disable() {
        DISABLED.store(true, Ordering::SeqCst);
    }

    pub fn enable() {
        DISABLED.store(false, Ordering::SeqCst);
    }

    pub fn disabled() -> bool {
        DISABLED.load(Ordering::SeqCst)
    }

    fn build_key(user_id: Option<i32>, limit_type: &str) -> String {
        match user_id {
            Some(id) => format!("{}:{}:{}", Self::key_prefix(), id, limit_type),
            None => format!("{}:{}", Self::key_prefix(), limit_type),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        user_id: Option<i32>,
        is_staff: bool,
        limit_type: String,
        max: isize,
        secs: isize,
        global: bool,
        aggressive: bool,
        error_code: Option<String>,
        apply_limit_to_staff: bool,
        staff_limit: Option<(isize, isize)>,
    ) -> Self {
        let key = Self::build_key(user_id, &limit_type);

        let mut final_max = max;
        let mut final_secs = secs;

        if is_staff && !apply_limit_to_staff {
            if let Some((s_max, s_secs)) = staff_limit {
                final_max = s_max;
                final_secs = s_secs;
            }
        }

        Self {
            max: final_max,
            secs: final_secs,
            user_id,
            is_staff,
            limit_type,
            key,
            global,
            aggressive,
            error_code,
            apply_limit_to_staff,
            staff_limit,
        }
    }

    pub fn prefixed_key(&self) -> String {
        if self.global {
            format!("GLOBAL::{}", self.key)
        } else {
            self.key.clone()
        }
    }

    pub async fn clear_all_global(conn: &mut Connection) -> Result<(), RateLimiterError> {
        let pattern = format!("GLOBAL::{}*", Self::key_prefix());
        let keys: Vec<String> = conn.keys(pattern).await?;
        if !keys.is_empty() {
            let _: () = conn.del(keys).await?;
        }
        Ok(())
    }

    pub async fn clear(&self, conn: &mut Connection) -> Result<(), RateLimiterError> {
        let _: () = conn.del(self.prefixed_key()).await?;
        Ok(())
    }

    pub async fn age_of_oldest(&self, conn: &mut Connection, now: isize) -> Result<isize, RateLimiterError> {
        let items: Vec<String> = conn.lrange(self.prefixed_key(), -1, -1).await?;
        let oldest = items.first().and_then(|s| s.parse::<isize>().ok()).unwrap_or(0);
        Ok(now - oldest)
    }

    pub async fn is_under_limit(&self, conn: &mut Connection) -> Result<bool, RateLimiterError> {
        let now = chrono::Utc::now().timestamp() as isize;
        let len: isize = conn.llen(self.prefixed_key()).await.unwrap_or(0);
        if len < self.max {
            return Ok(true);
        }

        let age = self.age_of_oldest(conn, now).await?;
        Ok(age >= self.secs)
    }

    pub fn rate_unlimited(&self) -> bool {
        if Self::disabled() {
            return true;
        }

        if self.is_staff && !self.apply_limit_to_staff && self.staff_limit.map(|s| s.0).is_none() {
            return true;
        }

        false
    }

    pub async fn can_perform(&self, conn: &mut Connection) -> Result<bool, RateLimiterError> {
        if self.rate_unlimited() {
            return Ok(true);
        }
        self.is_under_limit(conn).await
    }

    pub async fn seconds_to_wait(&self, conn: &mut Connection, now: isize) -> Result<isize, RateLimiterError> {
        let age = self.age_of_oldest(conn, now).await?;
        Ok(self.secs - age)
    }

    async fn rate_limiter_allowed(&self, conn: &mut Connection, now: isize) -> Result<bool, RateLimiterError> {
        let script_content = if self.aggressive {
            PERFORM_LUA_AGGRESSIVE
        } else {
            PERFORM_LUA
        };

        let script = redis::Script::new(script_content);
        let result: isize = script
            .key(self.prefixed_key())
            .arg(now)
            .arg(self.secs)
            .arg(self.max)
            .invoke_async(conn)
            .await?;

        Ok(result == 1)
    }

    pub async fn performed(&self, conn: &mut Connection, raise_error: bool) -> Result<bool, RateLimiterError> {
        if self.rate_unlimited() {
            return Ok(true);
        }

        let now = chrono::Utc::now().timestamp() as isize;

        let allowed = self.rate_limiter_allowed(conn, now).await?;

        if self.max <= 0 || !allowed {
            if raise_error {
                let wait = self.seconds_to_wait(conn, now).await.unwrap_or(self.secs);
                return Err(RateLimiterError::LimitExceeded {
                    seconds_to_wait: wait,
                    limit_type: self.limit_type.clone(),
                    error_code: self.error_code.clone(),
                });
            }
            Ok(false)
        } else {
            Ok(true)
        }
    }

    pub async fn rollback(&self, conn: &mut Connection) -> Result<(), RateLimiterError> {
        if Self::disabled() {
            return Ok(());
        }
        let _: Option<String> = conn.lpop(self.prefixed_key(), None).await?;
        Ok(())
    }

    pub async fn remaining(&self, conn: &mut Connection) -> Result<isize, RateLimiterError> {
        if self.is_staff {
            return Ok(self.max);
        }

        let arr: Vec<String> = conn.lrange(self.prefixed_key(), 0, self.max).await?;
        let t0 = chrono::Utc::now().timestamp() as isize;

        let mut valid_count = 0;
        for a in arr {
            if let Ok(val) = a.parse::<isize>() {
                if (t0 - val) <= self.secs {
                    valid_count += 1;
                }
            }
        }

        let remaining = self.max - valid_count as isize;
        Ok(std::cmp::max(0, remaining))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use deadpool_redis::{Config, Pool, Runtime};

    async fn create_pool() -> Pool {
        let cfg = Config::from_url("redis://127.0.0.1/");
        cfg.create_pool(Some(Runtime::Tokio1)).unwrap()
    }

    async fn build_limiter(max: isize, secs: isize, aggressive: bool) -> RateLimiter {
        RateLimiter::new(
            Some(1),
            false,
            "test_limit".to_string(),
            max,
            secs,
            false,
            aggressive,
            None,
            false,
            None,
        )
    }

    #[tokio::test]
    #[ignore = "Requires running Redis server"]
    async fn test_under_limit_and_over_limit() {
        let pool = create_pool().await;
        let mut conn = pool.get().await.unwrap();
        let _: () = redis::cmd("FLUSHALL").query_async(&mut conn).await.unwrap();

        let limiter = build_limiter(2, 10, false).await;
        limiter.clear(&mut conn).await.unwrap();

        assert_eq!(limiter.remaining(&mut conn).await.unwrap(), 2);

        assert_eq!(limiter.performed(&mut conn, false).await.unwrap(), true);
        assert_eq!(limiter.remaining(&mut conn).await.unwrap(), 1);

        assert_eq!(limiter.performed(&mut conn, false).await.unwrap(), true);
        assert_eq!(limiter.remaining(&mut conn).await.unwrap(), 0);

        assert_eq!(limiter.performed(&mut conn, false).await.unwrap(), false);

        limiter.clear(&mut conn).await.unwrap();
    }

    #[tokio::test]
    #[ignore = "Requires running Redis server"]
    async fn test_wait_time() {
        let pool = create_pool().await;
        let mut conn = pool.get().await.unwrap();
        let _: () = redis::cmd("FLUSHALL").query_async(&mut conn).await.unwrap();

        let limiter = build_limiter(1, 10, false).await;
        limiter.clear(&mut conn).await.unwrap();

        limiter.performed(&mut conn, false).await.unwrap();

        let wait_err = limiter.performed(&mut conn, true).await.unwrap_err();
        match wait_err {
            RateLimiterError::LimitExceeded { seconds_to_wait, .. } => {
                assert!(seconds_to_wait > 8 && seconds_to_wait <= 10);
            }
            _ => panic!("Expected LimitExceeded"),
        }

        limiter.clear(&mut conn).await.unwrap();
    }

    #[tokio::test]
    #[ignore = "Requires running Redis server"]
    async fn test_rollback() {
        let pool = create_pool().await;
        let mut conn = pool.get().await.unwrap();
        let _: () = redis::cmd("FLUSHALL").query_async(&mut conn).await.unwrap();

        let limiter = build_limiter(2, 10, false).await;
        limiter.clear(&mut conn).await.unwrap();

        limiter.performed(&mut conn, false).await.unwrap();
        assert_eq!(limiter.remaining(&mut conn).await.unwrap(), 1);

        limiter.rollback(&mut conn).await.unwrap();
        assert_eq!(limiter.remaining(&mut conn).await.unwrap(), 2);

        limiter.clear(&mut conn).await.unwrap();
    }

    #[tokio::test]
    #[ignore = "Requires running Redis server"]
    async fn test_aggressive_mode() {
        let pool = create_pool().await;
        let mut conn = pool.get().await.unwrap();
        let _: () = redis::cmd("FLUSHALL").query_async(&mut conn).await.unwrap();

        let limiter = build_limiter(1, 10, true).await;
        limiter.clear(&mut conn).await.unwrap();

        assert_eq!(limiter.performed(&mut conn, false).await.unwrap(), true, "first performed should be true");
        assert_eq!(limiter.performed(&mut conn, false).await.unwrap(), false, "second performed should be false");

        let len: isize = conn.llen(limiter.prefixed_key()).await.unwrap_or(0);
        assert_eq!(len, 2);

        limiter.clear(&mut conn).await.unwrap();
    }
}
