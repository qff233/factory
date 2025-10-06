use std::{ops::Deref, sync::Arc};

use sqlx::{prelude::FromRow, query, query_as};

use crate::{db_manager::DbManager, transport::prelude::Position};

pub type Result<T> = std::result::Result<T, sqlx::Error>;

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "nodetype")]
pub enum NodeType {
    #[sqlx(rename = "shipping_dock")]
    ShippingDock,
    #[sqlx(rename = "parking_station")]
    ParkingStation,
    #[sqlx(rename = "charging_station")]
    ChargingStation,
    #[sqlx(rename = "item_stocker")]
    ItemStocker,
    #[sqlx(rename = "fluid_stocker")]
    FluidStocker,
    #[sqlx(rename = "fork")]
    Fork,
}

#[derive(Debug)]
pub struct Node {
    pub id: i32,
    pub name: String,
    pub node_type: NodeType,
    pub position: Position,
    pub comment: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Path(Vec<Arc<Node>>);

impl Deref for Path {
    type Target = [Arc<Node>];
    fn deref(&self) -> &Self::Target {
        self.0.as_slice()
    }
}

#[derive(Debug, FromRow)]
struct Row {
    id: i32,
    name: String,
    #[sqlx(rename = "type")]
    node_type: NodeType,
    x: f64,
    y: f64,
    z: f64,
    comment: Option<String>,
}

#[derive(Debug)]
pub struct Graph {
    db: Arc<DbManager>,
}

impl Graph {
    pub async fn new(db: Arc<DbManager>) -> Self {
        let mut conn = db.track().await.unwrap();
        query("UPDATE edges SET is_lock = false;")
            .execute(&mut *conn)
            .await
            .unwrap();
        Self { db }
    }

    pub async fn find_shortest_node(&self, position: &Position) -> Result<Arc<Node>> {
        let mut conn = self.db.track().await?;
        let row = sqlx::query_as::<_, Row>(
            "SELECT id, name, type, comment, ST_X(geom) as x, ST_Y(geom) as y, ST_Z(geom) as z, ST_Distance(geom, ST_MakePoint($1,$2,$3)) as dist 
                FROM nodes ORDER by dist LIMIT 1;")
            .bind(position.0)
            .bind(position.1)
            .bind(position.2)
            .fetch_one(&mut *conn)
            .await?;
        Ok(Arc::new(Node {
            id: row.id,
            name: row.name,
            node_type: row.node_type,
            position: Position(row.x, row.y, row.z),
            comment: row.comment,
        }))
    }

    pub async fn lock_node(&self, node_id: i32) -> Result<()> {
        let mut conn = self.db.track().await?;
        query("UPDATE edges SET is_lock = true WHERE begin_node_id = $1 OR end_node_id = $1;")
            .bind(node_id)
            .execute(&mut *conn)
            .await?;
        Ok(())
    }

    pub async fn unlock_node(&self, node_id: i32) -> Result<()> {
        let mut conn = self.db.track().await?;
        query("UPDATE edges SET is_lock = false WHERE begin_node_id = $1 OR end_node_id = $1;")
            .bind(node_id)
            .execute(&mut *conn)
            .await?;
        Ok(())
    }

    pub async fn find_path(&self, begin_node_name: &str, end_node_name: &str) -> Result<Path> {
        let mut conn = self.db.track().await?;
        let rows = query_as::<_, Row>(
            "
                WITH path AS (
                SELECT * FROM
                    track.pgr_astar(
                    'SELECT 
                        e.id, 
                        e.begin_node_id AS source, 
                        e.end_node_id AS target, 
                        e.cost,
                        e.reverse_cost,
                        ST_X(n_src.geom) AS x1, 
                        ST_Y(n_src.geom) AS y1,
                        ST_X(n_tgt.geom) AS x2,
                        ST_Y(n_tgt.geom) AS y2
                    FROM edges AS e
                    JOIN nodes AS n_src ON e.begin_node_id = n_src.id
                    JOIN nodes AS n_tgt ON e.end_node_id = n_tgt.id
                    WHERE is_lock=false',
                        (SELECT id FROM nodes WHERE name = $1),
                        (SELECT id FROM nodes WHERE name = $2),
                        true)
                )
                SELECT
                    nodes.id, nodes.name, nodes.type, nodes.comment, ST_X(nodes.geom) as x, ST_Y(nodes.geom) as y, ST_Z(nodes.geom) as z
                FROM path
                JOIN nodes ON path.node = nodes.id;
                ",
            )
            .bind(begin_node_name)
            .bind(end_node_name)
            .fetch_all(&mut *conn)
            .await?;

        let mut path = Path(Vec::with_capacity(rows.len()));
        for row in rows {
            path.0.push(Arc::new(Node {
                id: row.id,
                name: row.name,
                node_type: row.node_type,
                position: Position(row.x, row.y, row.z),
                comment: row.comment,
            }));
        }
        Ok(path)
    }

    pub async fn find_path_by_type(
        &self,
        begin_node_name: &str,
        node_type: &NodeType,
    ) -> Result<Path> {
        let mut conn = self.db.track().await?;
        let rows = query_as::<_, Row>(
            "
            WITH current_node AS (
                SELECT id, geom FROM nodes WHERE name = $1
            ),
            shortest_node AS (
            SELECT nodes.id, ST_Distance(current_node.geom, nodes.geom) as dist
            FROM nodes, current_node
            WHERE type = $2
            ORDER by dist
            LIMIT 1
            ),
            path AS (
            SELECT * FROM
                track.pgr_astar(
                'SELECT 
                    e.id, 
                    e.begin_node_id AS source, 
                    e.end_node_id AS target, 
                    e.cost,
                    e.reverse_cost,
                    ST_X(n_src.geom) AS x1, 
                    ST_Y(n_src.geom) AS y1,
                    ST_X(n_tgt.geom) AS x2,
                    ST_Y(n_tgt.geom) AS y2
                FROM edges AS e
                JOIN nodes AS n_src ON e.begin_node_id = n_src.id
                JOIN nodes AS n_tgt ON e.end_node_id = n_tgt.id
                WHERE is_lock=false',
                    (SELECT id FROM current_node),
                    (SELECT id FROM shortest_node),
                    true
                )
            )
            SELECT
                nodes.id, nodes.name, nodes.type, nodes.comment, ST_X(nodes.geom) as x, ST_Y(nodes.geom) as y, ST_Z(nodes.geom) as z
            FROM path
            JOIN nodes ON path.node = nodes.id; 
        ",
        ).bind(begin_node_name)
        .bind(node_type)
        .fetch_all(&mut *conn)
        .await?;

        let mut path = Path(Vec::with_capacity(rows.len()));
        for row in rows {
            path.0.push(Arc::new(Node {
                id: row.id,
                name: row.name,
                node_type: row.node_type,
                position: Position(row.x, row.y, row.z),
                comment: row.comment,
            }));
        }
        Ok(path)
    }

    pub async fn find_parking_path(&self, from_node_name: &str) -> Result<Path> {
        self.find_path_by_type(from_node_name, &NodeType::ParkingStation)
            .await
    }

    pub async fn find_charging_path(&self, from_node_name: &str) -> Result<Path> {
        self.find_path_by_type(from_node_name, &NodeType::ChargingStation)
            .await
    }

    pub async fn find_shipping_dock_path(&self, from_node_name: &str) -> Result<Path> {
        self.find_path_by_type(from_node_name, &NodeType::ShippingDock)
            .await
    }
}

#[cfg(test)]
mod tests {
    use dotenvy::dotenv;
    use sqlx::postgres::PgPoolOptions;

    use super::*;

    async fn get_track_graph() -> Graph {
        dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        println!("Connecting to database: {}", database_url);
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to create pool");
        let db_manaer = DbManager::new(pool);
        Graph::new(db_manaer).await
    }

    #[tokio::test]
    async fn get_shortest_node() {
        let track_graph = get_track_graph().await;
        let node = track_graph
            .find_shortest_node(&Position(8.0, 8.0, 8.0))
            .await
            .unwrap();
        assert_eq!(node.id, 13);
    }

    #[tokio::test]
    async fn find_path() {
        let track_graph = get_track_graph().await;
        let path = track_graph.find_path("S2", "S1").await.unwrap();

        assert_eq!(path.0.get(0).unwrap().name, "S2");
        assert_eq!(path.0.get(1).unwrap().name, "A6");
        assert_eq!(path.0.get(2).unwrap().name, "A2");
        assert_eq!(path.0.get(3).unwrap().name, "A1");
        assert_eq!(path.0.get(4).unwrap().name, "A4");
        assert_eq!(path.0.get(5).unwrap().name, "A3");
        assert_eq!(path.0.get(6).unwrap().name, "S1");

        let path = track_graph
            .find_path_by_type("S3", &NodeType::ParkingStation)
            .await
            .unwrap();
        assert_eq!(path.0.get(0).unwrap().name, "S3");
        assert_eq!(path.0.get(1).unwrap().name, "A5");
        assert_eq!(path.0.get(2).unwrap().name, "A6");
        assert_eq!(path.0.get(3).unwrap().name, "P2");
    }
}
