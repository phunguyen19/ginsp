use toml::Table;

pub mod profile;

pub struct Config {
    config_content: Table,
}

impl Config {
    pub fn read_config_file() {
        let path_buf = home::home_dir().unwrap();

        let homedir = path_buf.to_str().unwrap();

        let content_string =
            std::fs::read_to_string(format!("{}/.ginsp/config.toml", homedir).as_str()).unwrap();

        let content_toml = toml::from_str::<Table>(content_string.as_str()).unwrap();

        println!("{:?}", content_toml);
    }
}
