use std::{
    fs::{read_to_string},
    io
};
use crate::data::{Command, Config, DataProvider, process_price};

mod data;

fn main() -> io::Result<()>{
    // read configuration
    let toml_config_str = read_to_string("./config.toml").unwrap();
    let conf: Config = toml::from_str(&toml_config_str).unwrap();

    // setup
    let mut count = 0;
    let mut last_bid =-1.0;
    let mut last_ask = -1.0;
    let mut command = Command::GetBoth;
    let mut last_quote = (0,-1.0,-1.0);
    let mut last_trade = (0,-1.0);

    let dp = DataProvider::new(&conf);
    if dp.is_none(){return Ok(());}
    let mut dp = dp.unwrap();

    // skip headers (opt.)
    if conf.quote_file_has_header {
        if dp.next_quote_row().is_none(){
            println!("Empty quote file. Program is closed.");
            return Ok(());
        }
    }
    if conf.trade_file_has_header {
        if dp.next_trade_row().is_none(){
            println!("Empty trade file. Program is closed.");
            return Ok(());
        }
    }

    // main loop
    loop{
        let mut check_spread_condition = true;
        // get new data
        match command{
            Command::GetBoth =>{
                last_quote = dp.next_quote_record();
                last_trade = dp.next_trade_record();
                if last_quote.1 < 0.0 || last_trade.1 < 0.0 {
                    println!("No data. Program is closed.");
                    break;
                }
                check_spread_condition = false;
            },
            Command::GetTrade =>{
                last_trade = dp.next_trade_record();
                if last_trade.1 < 0.0{ break; }
            },
            Command::GetQuote =>{
                last_quote = dp.next_quote_record();
                if last_quote.1 < 0.0{ break; }
            }
        }
        // do job
        let (incr,new_cmd) = apply_logic(&mut last_bid, &mut last_ask,
                    &mut last_quote, last_trade, check_spread_condition);
        count += incr;
        command = new_cmd;
    }

    // show result
    println!("Trades within spread : {}", count);
    Ok(())
}

fn apply_logic(mut last_bid: &mut f32, mut last_ask: &mut f32,
               last_quote: &(usize, f32, f32), last_trade: (usize, f32),
               check_spread_condition: bool) -> (usize,Command) {
    return if last_trade.0 < last_quote.0 {
        process_trade(*last_bid, *last_ask, last_trade, check_spread_condition)
    } else {
        (0, apply_new_quote(&mut last_bid, &mut last_ask, &last_quote))
    }
}
fn process_trade(last_bid: f32, last_ask: f32, last_trade: (usize, f32),
                check_spread_condition: bool) -> (usize,Command){
    return if check_spread_condition && last_ask > 0.0 {
        (process_price(last_trade.1, last_ask, last_bid),Command::GetTrade)
    } else { (0,Command::GetTrade) }
}
fn apply_new_quote(last_bid: &mut f32, last_ask: &mut f32,
                   last_quote: &(usize, f32, f32)) -> Command {
    *last_ask = last_quote.1;
    *last_bid = last_quote.2;
    Command::GetQuote
}
