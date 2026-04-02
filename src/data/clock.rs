use rsntp::AsyncSntpClient;
use chrono::{DateTime, Local};

pub async fn local_time() -> String {
    let client = AsyncSntpClient::new();
    let result = client.synchronize("pool.ntp.org").await.unwrap();

    let dt: DateTime<Local> = DateTime::from(result.datetime().into_chrono_datetime().unwrap());    

    dt.format("%H:%M:%S").to_string()              
} 
