use permutation_iterator::Permutor;
fn main() { for max in (1..=64).chain([65,127,128,129,255,256,257,1023,1024,1025]) { for key in 0..32u64 { let values: Vec<String> = Permutor::new_with_u64_key(max,key).map(|x| x.to_string()).collect(); println!("{}:{}:{}", max,key,values.join(",")); } } }
