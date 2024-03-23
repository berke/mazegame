use serde::{
    Deserialize,
    Serialize
};

const A:u32=1664525;
const B:u32=1013904223;

#[derive(Debug,Serialize,Deserialize)]
pub struct MiniRNG {
    q:u32
}
impl MiniRNG {
    pub fn new(seed:u32)->Self {
	let mut rng=MiniRNG{q:seed};
	let _ = rng.next();
	rng
    }
    pub fn next(&mut self)->u32 {
	let x=self.q;
	self.q=u32::wrapping_mul(A,u32::wrapping_add(self.q,B));
	x
    }
    pub fn uniform(&mut self)->f64 {
	let q1=self.next() as u64;
	let q2=self.next() as u64;
	let q=(q1<<16)|(q2>>16);
	let q=q as f64;
	q/((1u64<<48) as f64)
    }
    pub fn sample_u32(&mut self,upper:u32)->u32 {
	let x=self.uniform();
	((upper as f64)*x).trunc() as u32
    }
}

#[test]
fn test_uniform() {
    let mut rng=MiniRNG::new(1);
    for i in 0..1000 {
	let x=rng.uniform();
	println!("{}",x)
    }
}

#[test]
fn test_mini_rng_period() {
    let q0=1_u32;
    let mut rng=MiniRNG::new(q0);
    let mut n=0_u64;
    loop {
	let q=rng.next();
	n=n+1;
	if q==q0 { break; }
    }
    println!("Period: {}",n);
}
