use std::{
    fs::{read_to_string},
    io
};
use crate::data::{Command, Config, Converter, process_price, QuoteInfo, TradeInfo};
use crate::two_files_iterator::TwoFilesIterator;

mod data;
mod two_files_iterator;

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

    let qin = QuoteInfo{ ask_offset: conf.quote_ask_offset, bid_offset: conf.quote_bid_offset };
    let tin = TradeInfo{ price_offset: conf.trade_price_offset };

    let tfi = TwoFilesIterator::new(conf.quote_file_path, conf.trade_file_path,
                                    &tin as &dyn Converter<(usize,f32)>,
                                    &qin as &dyn Converter<(usize,f32,f32)>);
    if tfi.is_none(){
        println!("Impossible to create data provider (check file paths, please). Program is closed.");
        return Ok(());
    }
    let mut tfi = tfi.unwrap();

    // skip headers (opt.)
    if conf.quote_file_has_header {
        if tfi.next_quote_row().is_none(){
            println!("Empty quote file. Program is closed.");
            return Ok(());
        }
    }
    if conf.trade_file_has_header {
        if tfi.next_trade_row().is_none(){
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
                let lq = tfi.next_quote_record();
                let lt = tfi.next_trade_record();
                if lq.is_none()|| lt.is_none() {
                    println!("No data. Program is closed.");
                    break;
                }
                last_quote = lq.unwrap();
                last_trade = lt.unwrap();
                check_spread_condition = false;
            },
            Command::GetTrade =>{
                let lt = tfi.next_trade_record();
                if lt.is_none(){ break; }
                last_trade = lt.unwrap();
            },
            Command::GetQuote =>{
                let lq = tfi.next_quote_record();
                if lq.is_none(){ break; }
                last_quote = lq.unwrap();
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
