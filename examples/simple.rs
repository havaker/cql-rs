extern crate scylla;

use std::io;

use scylla::Connection;
use scylla::Query;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut conn = Connection::new("172.17.0.4:9042").await?;

    let query1 = Query::new("INSERT INTO ks.t(a,b,c) VALUES (1,2,'abc')");
    let query2 = Query::new("INSERT INTO ks.t(a,b,c) VALUES (4,5,'def')");

    conn.query(query1).await.unwrap();
    conn.query(query2).await.unwrap();

    return Ok(());
}
