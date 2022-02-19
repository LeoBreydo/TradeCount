use std::fs::read_to_string;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub quote_file_path: String,
    pub trade_file_path: String,
    pub quote_file_has_header: bool,
    pub trade_file_has_header: bool,
    pub trade_price_offset: usize,
    pub quote_ask_offset: usize,
    pub quote_bid_offset: usize
}

impl Config{
    pub fn from_file(path: &str) -> Result<Self,&'static str>{
        match read_to_string(path){
            Err(..) =>{ return Err("Can't read config. file. Program is closed."); },
            Ok(s) => match toml::from_str(&s) {
                Err(..) => { return Err("Can't parse config. file. Program is closed."); },
                Ok(c) => Ok(c)
            }
        }
    }
}