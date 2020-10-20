extern crate scylla;

use std::io;

use scylla::Connection;
use scylla::Query;

#[tokio::main]
async fn main() -> io::Result<()> {
    let conn = Connection::new("127.0.0.1:9042").await?;

    let query = Query::new("hej");
    conn.query(query).await.unwrap();
    return Ok(());
}
