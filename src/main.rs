use std::{
    fs::{File, read_to_string},
    io::{
        self,
        BufRead,
        BufReader
    }
};
use crate::data::{Command, Config, process_price, process_quote_record, process_trade_record};

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

    // get iterators
    let quote_file = File::open(conf.quote_file_path)?;
    let quote_reader = BufReader::new(quote_file);
    let mut quote_iterator = quote_reader.lines();

    let trade_file = File::open(conf.trade_file_path)?;
    let trade_reader = BufReader::new(trade_file);
    let mut trade_iterator = trade_reader.lines();

    // skip headers (opt.)
    if conf.quote_file_has_header {
        let r = quote_iterator.next();
        if r.is_none(){
            println!("Empty quote file. Program is closed.");
            return Ok(());
        }
    }
    if conf.trade_file_has_header {
        let r = trade_iterator.next();
        if r.is_none(){
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
                let q = quote_iterator.next();
                let t = trade_iterator.next();
                if q.is_none() || t.is_none() {
                    println!("No data. Program is closed.");
                    return Ok(());
                }
                last_quote = process_quote_record(q.unwrap().unwrap(), conf.quote_ask_offset,conf.quote_bid_offset);
                last_trade = process_trade_record(t.unwrap().unwrap(), conf.trade_price_offset);
                if last_quote.1 < 0.0 || last_trade.1 < 0.0 {
                    println!("No data. Program is closed.");
                    return Ok(());
                }
                check_spread_condition = false;
            },
            Command::GetTrade =>{
                let t = trade_iterator.next();
                if t.is_none(){
                    break;
                }
                last_trade = process_trade_record(t.unwrap().unwrap(), conf.trade_price_offset);
                if last_trade.1 < 0.0{
                    break;
                }
            },
            Command::GetQuote =>{
                let q = quote_iterator.next();
                if q.is_none(){
                    break;
                }
                last_quote = process_quote_record(q.unwrap().unwrap(), conf.quote_ask_offset,conf.quote_bid_offset);
                if last_quote.1 < 0.0{
                    break;
                }
            }
        }
        // do job
        apply_logic(&mut count, &mut last_bid, &mut last_ask, &mut command,
                    &mut last_quote, last_trade, check_spread_condition);
    }

    // show result
    println!("Trades within spread : {}", count);
    Ok(())
}

fn apply_logic(mut count: &mut usize, mut last_bid: &mut f32, mut last_ask: &mut f32,
               mut command: &mut Command, mut last_quote: &mut (usize, f32, f32),
               last_trade: (usize, f32), check_spread_condition: bool) {
    if last_trade.0 < last_quote.0 {
        process_trade(&mut count, *last_bid, *last_ask, &mut command, last_trade, check_spread_condition);
    } else {
        apply_new_quote(&mut last_bid, &mut last_ask, &mut command, &mut last_quote);
    }
}
fn process_trade(count: &mut usize, last_bid: f32,
                 last_ask: f32, command: &mut Command, last_trade: (usize, f32),
                check_spread_condition: bool) {
    *command = Command::GetTrade;
    if check_spread_condition && last_ask > 0.0 {
        *count += process_price(last_trade.1, last_ask, last_bid);
    }
}
fn apply_new_quote(last_bid: &mut f32, last_ask: &mut f32, command: &mut Command, last_quote: &mut (usize, f32, f32)) {
    *command = Command::GetQuote;
    *last_ask = last_quote.1;
    *last_bid = last_quote.2;
}
