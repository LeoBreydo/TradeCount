use std::fs::File;
use std::io::{BufReader, Lines};
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

pub enum Command{
    GetBoth,
    GetQuote,
    GetTrade
}

pub fn process_price(price:f32, ask:f32, bid: f32) -> usize{
    return if price > bid && price < ask { 1 } else { 0 }
}
// -> (ts, price)
pub fn process_trade_record(record: String, price_offset:usize) -> (usize,f32){
    let v: Vec<&str> = record.split(",").collect();
    if v.len() < price_offset+1{
        println!("Corrupted data detected");
        return (0,-1.0);
    }
    let time = get_millis(&v[0]);
    if time.is_none(){
        println!("Corrupted data detected");
        return (0,-1.0);
    }
    let price = v[price_offset].parse::<f32>();
    if price.is_err(){
        println!("Corrupted data detected");
        return (0,-1.0);
    }
    (time.unwrap(), price.unwrap())
}
// -> (ts, ask,bid)
pub fn process_quote_record(record: String, ask_offset:usize, bid_offset:usize) -> (usize,f32,f32){
    let v: Vec<&str> = record.split(",").collect();
    if v.len() < bid_offset+1{
        println!("Corrupted data detected");
        return (0,-1.0,-1.0);
    }
    let time = get_millis(&v[0]);
    if time.is_none(){
        println!("Corrupted data detected");
        return (0,-1.0,-1.0);
    }
    let ask = v[ask_offset].parse::<f32>();
    if ask.is_err(){
        println!("Corrupted data detected");
        return (0,-1.0,-1.0);
    }
    let bid = v[bid_offset].parse::<f32>();
    if bid.is_err(){
        println!("Corrupted data detected");
        return (0,-1.0,-1.0);
    }
    (time.unwrap(),ask.unwrap(),bid.unwrap())
}
const H_FACTOR:usize = 3600000;
const M_FACTOR:usize = 60000;
const S_FACTOR:usize = 1000;
fn get_millis(field:&str)->Option<usize>{
    let v:Vec<&str> = field.split(":").collect();
    if v.len() != 3{
        return None;
    }
    let vv:Vec<&str> = v[2].split(".").collect();
    if vv.len() != 2{
        return None;
    }
    let hr = v[0].parse::<usize>();
    if hr.is_err(){
        return None;
    }
    let h = hr.unwrap()* H_FACTOR;
    let mr = v[1].parse::<usize>();
    if mr.is_err(){
        return None;
    }
    let m = mr.unwrap()* M_FACTOR;
    let sr = vv[0].parse::<usize>();
    if sr.is_err(){
        return None;
    }
    let s = sr.unwrap()* S_FACTOR;
    let msr = vv[1].parse::<usize>();
    if msr.is_err(){
        return None;
    }
    let ms = msr.unwrap();
    Some(h+m+s+ms)
}

pub struct DataProvider<'a>{
    quote_iter : &'a mut Lines<BufReader<File>>,
    trade_iter : &'a mut Lines<BufReader<File>>,
    trade_price_offset: usize,
    quote_ask_offset: usize,
    quote_bid_offset: usize

}
impl<'a> DataProvider<'a> {
    pub fn new(qi:&'a mut Lines<BufReader<File>>, ti:&'a mut Lines<BufReader<File>>,
               trade_price_offset:usize, quote_ask_offset:usize, quote_bid_offset:usize) -> Self{
        Self{ quote_iter: qi, trade_iter: ti, trade_price_offset, quote_ask_offset, quote_bid_offset }
    }
    pub fn next_quote_row(&mut self) -> Option<std::io::Result<String>> {
        self.quote_iter.next()
    }
    pub fn next_trade_row(&mut self) -> Option<std::io::Result<String>> {
        self.trade_iter.next()
    }
    pub fn next_quote_record(&mut self) -> (usize,f32,f32){
        let q = self.quote_iter.next();
        return if q.is_none() { (0, -1.0, -1.0) } else {
            process_quote_record(q.unwrap().unwrap(),
                                 self.quote_ask_offset, self.quote_bid_offset)
        }
    }
    pub fn next_trade_record(&mut self) -> (usize,f32){
        let q = self.trade_iter.next();
        return if q.is_none() { (0, -1.0) } else {
            process_trade_record(q.unwrap().unwrap(), self.trade_price_offset)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn millis_test() {
        let s = "01:01:01.900";
        let v:Vec<&str> = s.split(":").collect();
        let vv:Vec<&str> = v[2].split(".").collect();
        let h = v[0].parse::<usize>().unwrap()*3600000;
        let m:usize = v[1].parse::<usize>().unwrap()*60000;
        let s:usize = vv[0].parse::<usize>().unwrap()*1000;
        let ms:usize = vv[1].parse::<usize>().unwrap();
        assert_eq!(3600000+60000+1000+900, h+m+s+ms);
    }
}

