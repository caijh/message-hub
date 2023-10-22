use config::Config;
use configuration::Database;
use rbatis::RBatis;
use rbdc_mysql::driver::MysqlDriver;


pub async fn init_rbatis(rbatis: &RBatis, config: &Config) -> Result<(), rbatis::Error> {
    let db = config.get::<Database>("database").expect("database properties load fail.");
    let db_url = db.kind
        + "://"
        + &db.user
        + ":"
        + &db.password
        + "@"
        + &db.host
        + ":"
        + &db.port.to_string()
        + "/"
        + &db.name;

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
