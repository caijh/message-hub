use config::Config;
use rbatis::RBatis;
use rbdc_mysql::driver::MysqlDriver;


pub async fn init_rbatis(rbatis: &RBatis, config: &Config) -> Result<(), rbatis::Error> {
    let db_host = config.get_string("database_host").unwrap();
    let db_port: i64 = config.get_int("database_port").unwrap();
    let db_user = config.get_string("database_user").unwrap();
    let db_password = config.get_string("database_password").unwrap();
    let db_name = config.get_string("database_name").unwrap();
    let db_type = config.get_string("database_type").unwrap();
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
    pub async fn new(config: &Config) -> Self {
        let rb = RBatis::new();
        init_rbatis(&rb, config).await.expect("init database error");
        DatabaseService { rb }
    }

    pub fn dao(&self) -> &RBatis {
        &self.rb
    }
}
