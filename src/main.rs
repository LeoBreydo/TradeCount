use crate::configuration::Config;
use crate::converters::{QuoteInfo,TradeInfo};
use crate::matcher::{Matcher, QuoteToTradeMatcher};
use crate::two_files_iterator::TwoFilesIterator;

mod converters;
mod two_files_iterator;
mod configuration;
mod matcher;

fn main() {
    // read configuration
    let conf: Config = match Config::from_file("./config.toml"){
        Err(msg) =>{
            println!("{}",msg);
            return;
        },
        Ok(c) => c
    };

    // setup
    let mut count = 0;
    let mut last_ask_bid:(f32,f32) = (-1.0,-1.0);
    let mut command = Command::GetBoth;
    let mut last_quote = (0,-1.0,-1.0);
    let mut last_trade = (0,-1.0);
    let qtm = QuoteToTradeMatcher::default();

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
        let (incr,new_cmd) = apply_logic(&mut last_ask_bid,
                    &mut last_quote, last_trade, qtm, check_spread_condition);
        count += incr;
        command = new_cmd;
    }

    // show result
    println!("Trades within spread : {}", count);
}

enum Command{
    GetBoth,
    GetQuote,
    GetTrade
}

fn apply_logic(mut last_ask_bid: &mut (f32,f32),
               last_quote: &(usize, f32, f32), last_trade: (usize, f32),
               qtm: impl Matcher<(f32,f32),(usize,f32)>,
               check_spread_condition: bool) -> (usize,Command) {
    return if last_trade.0 < last_quote.0 {
        process_trade(*last_ask_bid, last_trade, qtm, check_spread_condition)
    } else {
        (0, apply_new_quote(&mut last_ask_bid, &last_quote))
    }
}
fn process_trade(last_ask_bid: (f32,f32), last_trade: (usize, f32),
                 qtm: impl Matcher<(f32,f32),(usize,f32)>,
                 check_spread_condition: bool) -> (usize,Command){
    return if check_spread_condition && last_ask_bid.1 > 0.0 {
        (if qtm.is_matched(&last_ask_bid,&last_trade){1}else{0},Command::GetTrade)
    } else { (0,Command::GetTrade) }
}
fn apply_new_quote(last_ask_bid: &mut (f32,f32),
                   last_quote: &(usize, f32, f32)) -> Command {
    *last_ask_bid = (last_quote.1,last_quote.2);
    Command::GetQuote
}

