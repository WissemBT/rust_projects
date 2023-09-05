/** fibo recursive
 * fn fibo(n: u32) -> u32{
    
    if n==0{
         0
    }
    else if n==1{
        1
    }
    else{
    fibo(n-1)+fibo(n-2)
    }
} **/
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author="Wissem BEN BETTAIEB",version="0.1",about="Compute Fibonacci suite values")]
struct Args {
    ///The maximal number to print the fibo value of
    #[clap(short='n',long="number")]
    number : u32,
    ///Print intermediate values
    #[clap(short='v',long="verbose")]
    verbose: bool,
    ///The minimum number to compute
    #[clap(short='m',long="min",default_value_t=0)]
    min: u32,

}
fn fibo(n: u32) -> Option<u32>{
    
    if n==0 {return Some(0);}
    if n==1 {return Some(1);}
    
    let mut old_val:Option<u32> =Some(0);
    let mut val : Option<u32> =Some(1);
    let mut res : Option<u32> =Some(0);
    for _ in 2..=n{
        res=old_val.unwrap().checked_add(val.unwrap());
        old_val =val;
        val = res;
    }
    res
}


fn main() {
    let args= Args::parse();
    
    if args.verbose {
    for i in args.min..=args.number{ 
        match fibo(i) {
            Some(res) => println!("fibo({i})= {res}"),
            None => break,
        }
    }}
    else{
        match fibo(args.number){
            Some(res) => println!("fibo({})={res}",args.number),
            None => println!("the result can't fit in u32"),
        }
    }
}
