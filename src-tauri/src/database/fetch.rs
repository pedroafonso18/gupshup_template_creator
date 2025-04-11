use sqlx::{Error, PgConnection, Row};

#[derive(Debug)]
pub struct ConnectionData {
    pub id: i32,
    pub source_name: Option<String>,
    pub disparos_dia: i32,
    pub qualidade: Option<String>,
    pub restriction: Option<String>,
    pub issues: i32,
    pub msg_limit: Option<String>,
    pub app_id: Option<String>,
    pub ultima_issue_dia: Option<String>,
    pub facebook_id: Option<String>,
    pub facebook_token: Option<String>,
}

pub async fn fetch_connections(conn: &mut PgConnection) -> Result<Vec<ConnectionData>,Error> {
    println!("Executing database query: SELECT * FROM parametros");
    let query = String::from(
        r#"SELECT * FROM parametros"#
    );

    let rows = sqlx::query(&query)
        .fetch_all(conn)
        .await?;
    
    println!("Query executed successfully, fetched {} rows", rows.len());
    
    let mut connections_data: Vec<ConnectionData> = Vec::with_capacity(rows.len());

    for row in rows {
        let get_opt_string = |field: &str| -> Option<String> {
            row.try_get(field).ok()
        };

        let get_opt_i32 = |field: &str| -> Option<i32> {
            row.try_get(field).ok()
        };

        connections_data.push(ConnectionData {
            id: row.try_get("id").unwrap_or(0),
            source_name: get_opt_string("source_name"),
            disparos_dia: get_opt_i32("disparos_dia").unwrap_or(0),
            qualidade: get_opt_string("qualidade"),
            restriction: get_opt_string("restriction"),
            issues: get_opt_i32("issues").unwrap_or(0),
            msg_limit: get_opt_string("msg_limit"),
            app_id: get_opt_string("app_id"),
            ultima_issue_dia: get_opt_string("ultima_issue_dia"),
            facebook_id: get_opt_string("facebook_id"),
            facebook_token: get_opt_string("facebook_token"),
        });   
    }
    println!("Processed {} connection records", connections_data.len());
    Ok(connections_data)
}