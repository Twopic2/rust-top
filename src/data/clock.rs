use rsntp::AsyncSntpClient;
use chrono::{DateTime, Local};

pub async fn local_time() -> String {
    let client = AsyncSntpClient::new();
    if let Ok(result) = client.synchronize("pool.ntp.org").await {
        if let Ok(chrono_dt) = result.datetime().into_chrono_datetime() {
            let dt: DateTime<Local> = DateTime::from(chrono_dt);
            return dt.format("%H:%M:%S").to_string();
        }
    }

    Local::now().format("%H:%M:%S").to_string()              
} 
