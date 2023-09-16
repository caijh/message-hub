use rbatis::RBatis;
use rbdc_mysql::driver::MysqlDriver;

use crate::config::CONFIG;

pub async fn init_rbatis(rbatis: &RBatis) -> Result<(), rbatis::Error> {
    let db_host = CONFIG.database_host.clone();
    let db_port: u16 = CONFIG.database_port;
    let db_user = CONFIG.database_user.clone();
    let db_password = CONFIG.database_password.clone();
    let db_name = CONFIG.database_name.clone();
    let db_type = CONFIG.database_type.clone();
    let db_url = db_type.to_string()
        + "://"
        + &db_user
        + ":"
        + &db_password
        + "@"
        + &db_host
        + ":"
        + &db_port.to_string()
        + "/"
        + &db_name;

    rbatis.init(MysqlDriver {}, &db_url).unwrap();

    // test connection
    let _ = rbatis.query("select 1", vec![]).await?;
    Ok(())
}

pub struct DatabaseService {
    rb: RBatis,
}

impl DatabaseService {
    pub async fn new() -> Self {
        let rb = RBatis::new();
        init_rbatis(&rb).await.expect("init database error");
        DatabaseService { rb }
    }

    pub fn dao(&self) -> &RBatis {
        &self.rb
    }
}
