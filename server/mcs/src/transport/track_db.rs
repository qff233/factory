use std::sync::Arc;

use sqlx::{
    postgres::{self, PgPool},
    prelude::FromRow,
};

#[derive(Debug)]
pub struct Position(f64, f64, f64);

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        (self.0 - other.0).abs() < 0.1
            && (self.1 - other.1).abs() < 0.1
            && (self.2 - other.2).abs() < 0.1
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Side {
    NegY,
    PosY,
    NegZ,
    PosZ,
    NegX,
    PosX,
}

impl From<&str> for Side {
    fn from(value: &str) -> Self {
        match value {
            "negy" => Self::NegY,
            "posy" => Self::PosY,
            "negz" => Self::NegZ,
            "posz" => Self::PosZ,
            "negx" => Self::NegX,
            "posx" => Self::PosX,
            _ => panic!("no such side, {}", value),
        }
    }
}

#[derive(Debug)]
enum NodeType {
    ParkingStation,
    ChargingStation,
    ItemStocker(Side),
    FluidStocker(Side),
    Fork,
}

#[derive(Debug, FromRow)]
struct NodeRow {
    id: u32,
    name: String,
    #[sqlx(rename = "type")]
    node_type: String,
    side: Option<String>,
    position: Position,
    comment: String,
}

#[derive(Debug)]
struct Node {
    id: u32,
    name: String,
    node_type: NodeType,
    position: Position,
    comment: String,
}

struct Graph {
    pg_pool: Arc<PgPool>,
}

impl Graph {
    fn new(pg_pool: Arc<PgPool>) -> Self {
        // TODO check db has same table
        Self { pg_pool }
    }

    pub async fn get_shortest_node(&self, position: &Position) -> i32 {
        #[derive(Debug, FromRow)]
        struct Row {
            id: i32,
            name: String,
            x: f64,
            y: f64,
            z: f64,
            dist: f64,
        }
        let mut conn = self.pg_pool.acquire().await.unwrap();
        sqlx::query("SET search_path TO track;")
            .execute(&mut *conn)
            .await
            .unwrap();
        let row = sqlx::query_as::<_,Row>(
            "select id, name, ST_X(geom) as x, ST_Y(geom) as y, ST_X(geom) as z, ST_Distance(geom, ST_MakePoint($1,$2,$3)) as dist FROM nodes ORDER by dist LIMIT 1;")
            .bind(position.0)
            .bind(position.1)
            .bind(position.2)
            .fetch_one(&mut *conn)
            .await.unwrap();
        println!("{:#?}", 11);
        row.id
    }
}

#[cfg(test)]
mod tests {
    use dotenvy::dotenv;
    use sqlx::postgres::PgPoolOptions;
    use std::sync::Arc;

    use crate::transport::track_db::{Graph, Position};

    #[tokio::test]
    async fn get_shortest_node() {
        dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        println!("Connecting to database: {}", database_url);
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to create pool");
        let track = Graph::new(Arc::new(pool));
        let node_id = track.get_shortest_node(&Position(8.0, 8.0, 8.0)).await;
        assert_eq!(node_id, 5);
    }
}
