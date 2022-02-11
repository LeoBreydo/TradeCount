use std::fs::{File, read_to_string};
use std::io;
use std::io::{BufRead, BufReader};
use crate::data::{Command, Config, process_price, process_quote_record, process_trade_record};

mod data;

fn main() -> io::Result<()>{
    let toml_config_str = read_to_string("./config.toml").unwrap();
    let conf: Config = toml::from_str(&toml_config_str).unwrap();

    let mut count = 0;
    let mut last_bid =-1.0;
    let mut last_ask = -1.0;
    let mut command = Command::GetBoth;
    let mut last_quote = (0,-1.0,-1.0);
    let mut last_trade = (0,-1.0);

    let quote_file = File::open(conf.quote_file_path)?;
    let quote_reader = BufReader::new(quote_file);
    let mut quote_iterator = quote_reader.lines();

    let trade_file = File::open(conf.trade_file_path)?;
    let trade_reader = BufReader::new(trade_file);
    let mut trade_iterator = trade_reader.lines();

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

    loop{
        match command{
            Command::GetBoth =>{
                let q = quote_iterator.next();
                if q.is_none(){
                    println!("No quotes. Program is closed.");
                    return Ok(());
                }
                let t = trade_iterator.next();
                if t.is_none(){
                    println!("No trades. Program is closed.");
                    return Ok(());
                }
                last_quote = process_quote_record(q.unwrap().unwrap(),
                                                  conf.quote_ask_offset,conf.quote_bid_offset);
                last_trade = process_trade_record(t.unwrap().unwrap(), conf.trade_price_offset);
                if last_quote.1 < 0.0{
                    println!("No quotes. Program is closed.");
                    return Ok(());
                }
                if last_trade.1 < 0.0{
                    println!("No trades. Program is closed.");
                    return Ok(());
                }

                if last_trade.0 < last_quote.0{
                    command = Command::GetTrade;
                    continue;
                }
                else{
                    last_ask = last_quote.1;
                    last_bid = last_quote.2;
                    command = Command::GetQuote;
                    continue;
                }

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
                if last_trade.0 < last_quote.0{
                    command = Command::GetTrade;
                    if last_ask > 0.0{
                        count += process_price(last_trade.1, last_ask, last_bid);
                    }
                    continue;
                }
                else{
                    command = Command::GetQuote;
                    last_ask = last_quote.1;
                    last_bid = last_quote.2;
                    continue;
                }
            },
            Command::GetQuote =>{
                let q = quote_iterator.next();
                if q.is_none(){
                    break;
                }
                last_quote = process_quote_record(q.unwrap().unwrap(),
                                                  conf.quote_ask_offset,conf.quote_bid_offset);
                if last_quote.1 < 0.0{
                    break;
                }
                if last_trade.0 < last_quote.0{
                    command = Command::GetTrade;
                    if last_ask > 0.0{
                        count += process_price(last_trade.1, last_ask, last_bid);
                    }
                    continue;
                }
                else{
                    command = Command::GetQuote;
                    last_ask = last_quote.1;
                    last_bid = last_quote.2;
                    continue;
                }
            }
        }
    }
    println!("Trades within spread : {}", count);
    Ok(())
}
