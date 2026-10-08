//! Machine-readable primitive probe for independent exact-rational qualification.
//! Input: one line `id op lo_bits hi_bits [other_lo_bits other_hi_bits]`.
//! Bits are 16 hex digits. Output is one JSON object per input, echoing the
//! inputs and either result bits or an explicit refusal. No stepping occurs.
#[path = "../experiments/outward.rs"]
mod outward;
use std::io::{self, BufRead};
use outward::Interval;
fn decode(x: &str) -> f64 { f64::from_bits(u64::from_str_radix(x,16).expect("hex binary64 bits")) }
fn main() {
    outward::ensure_supported().expect("supported default arithmetic environment");
    for line in io::stdin().lock().lines() {
        let line = line.expect("stdin");
        let fields: Vec<_> = line.split_whitespace().collect();
        assert!(fields.len() == 4 || fields.len() == 6, "probe record shape");
        let id = fields[0];
        assert!(id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'));
        let op = fields[1];
        assert!(matches!(op,"add"|"sub"|"mul"|"div"|"neg"|"abs"|"square"));
        let a = Interval::new(decode(fields[2]),decode(fields[3]));
        let binary = matches!(op,"add"|"sub"|"mul"|"div");
        assert_eq!(fields.len(), if binary {6} else {4});
        let result = a.and_then(|a| {
            if binary {
                let b = Interval::new(decode(fields[4]),decode(fields[5]))?;
                match op { "add"=>a.add(b), "sub"=>a.sub(b), "mul"=>a.mul(b), "div"=>a.div(b), _=>unreachable!() }
            } else { match op {"neg"=>Ok(a.neg()),"abs"=>a.abs(),"square"=>a.square(),_=>unreachable!()} }
        });
        let b_json = if binary {format!(",\"b\":[\"{}\",\"{}\"]",fields[4],fields[5])} else {String::new()};
        let result_json = match result {
            Ok(x) => format!("\"result\":[\"{:016x}\",\"{:016x}\"]",x.lo.to_bits(),x.hi.to_bits()),
            Err(e) => format!("\"error\":\"{e:?}\""),
        };
        println!("{{\"schema\":\"rheon-outward-probe-v1\",\"id\":\"{id}\",\"op\":\"{op}\",\"a\":[\"{}\",\"{}\"]{b_json},{result_json}}}",fields[2],fields[3]);
    }
}
