use crate::db::{save_metric, DbPool};
use rand::Rng;
use tokio::time::{interval, Duration};

pub fn init_metrics(pool: DbPool) {
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(5));
        loop {
            ticker.tick().await;
            let (cpu, mem, users, rps) = {
                let mut rng = rand::thread_rng();
                (
                    rng.gen_range(10.0..90.0),
                    rng.gen_range(20.0..80.0),
                    rng.gen_range(100.0..1000.0),
                    rng.gen_range(50.0..500.0),
                )
            };
            let _ = save_metric(&pool, cpu, mem, users, rps).await;
        }
    });
}
