use std::fs::read_to_string;
use crate::data::{Command, Config, QuoteInfo, TradeInfo};
use crate::two_files_iterator::TwoFilesIterator;

mod data;
mod two_files_iterator;

fn main() {
    // read configuration
    let toml_config_str = match read_to_string("./config.toml"){
        Err(..) =>{
            println!("Can't read config. file. Program is closed.");
            return;
        },
        Ok(s) => s
    };
    let conf: Config = match toml::from_str(&toml_config_str) {
        Err(..) => {
            println!("Can't parse config. file. Program is closed.");
            return;
        },
        Ok(c) => c
    };

    // setup
    let mut count = 0;
    let mut last_bid =-1.0;
    let mut last_ask = -1.0;
    let mut command = Command::GetBoth;
    let mut last_quote = (0,-1.0,-1.0);
    let mut last_trade = (0,-1.0);

    let qin = QuoteInfo{ ask_offset: conf.quote_ask_offset, bid_offset: conf.quote_bid_offset };
    let tin = TradeInfo{ price_offset: conf.trade_price_offset };

    let mut tfi = match TwoFilesIterator::new(conf.quote_file_path, conf.trade_file_path, tin, qin){
        None => {
            println!("Impossible to create data provider (check file paths, please). Program is closed.");
            return;
        },
        Some(it) => it
    };

    // skip headers (opt.)
    if conf.quote_file_has_header {
        if tfi.next_quote_row().is_none(){
            println!("Empty quote file. Program is closed.");
            return;
        }
    }
    if conf.trade_file_has_header {
        if tfi.next_trade_row().is_none(){
            println!("Empty trade file. Program is closed.");
            return;
        }
    }

    // main loop
    loop{
        let mut check_spread_condition = true;
        // get new data
        match command{
            Command::GetBoth =>{
                last_quote = match tfi.next_quote_record(){
                    None =>{
                        println!("No data. Program is closed.");
                        break;
                    },
                    Some(lq) => lq
                };
                last_trade = match tfi.next_trade_record(){
                    None =>{
                        println!("No data. Program is closed.");
                        break;
                    },
                    Some(lt) => lt
                };
                check_spread_condition = false;
            },
            Command::GetTrade =>{
                last_trade = match tfi.next_trade_record(){
                    None => { break; },
                    Some(lt) => lt
                };
            },
            Command::GetQuote =>{
                last_quote = match tfi.next_quote_record(){
                    None => { break; },
                    Some(lq) => lq
                };
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
}

fn process_price(price:f32, ask:f32, bid: f32) -> usize{
    return if price > bid && price < ask { 1 } else { 0 }
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

