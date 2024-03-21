use std::ops::{
    Index,
    IndexMut
};

#[derive(Debug)]
pub struct A2<T> {
    d0:isize,
    d1:isize,
    data:Vec<T>,
    out:T
}

impl<T> A2<T> where T:Copy {
    pub fn new((d0,d1):(isize,isize),out:T)->Self {
	let m = (d0*d1) as usize;
	let d0 = d0 as isize;
	let d1 = d1 as isize;
	let data = vec![out;m];
	Self {
	    d0,
	    d1,
	    data,
	    out
	}
    }

    pub fn dims(&self)->(isize,isize) {
	(self.d0,self.d1)
    }

    pub fn as_slice(&self)->&[T] {
	&self.data
    }
}

impl<T> Index<[isize;2]> for A2<T> {
    type Output = T;

    fn index(&self,[i0,i1]:[isize;2])->&T {
	if 0 <= i0 && i0 < self.d0 &&
	    0 <= i1 && i1 < self.d1 {
		&self.data[(i0*self.d1 + i1) as usize]
	    } else {
		&self.out
	    }
    }
}

impl<T> IndexMut<[isize;2]> for A2<T> {
    fn index_mut(&mut self,[i0,i1]:[isize;2])->&mut T {
	if 0 <= i0 && i0 < self.d0 &&
	    0 <= i1 && i1 < self.d1 {
		&mut self.data[(i0*self.d1 + i1) as usize]
	    } else {
		panic!("Index {},{} out of bounds",i0,i1);
	    }
    }
}

impl<T> Index<[usize;2]> for A2<T> {
    type Output = T;

    fn index(&self,[i0,i1]:[usize;2])->&T {
	if i0 < self.d0 as usize && i1 < self.d1 as usize {
		&self.data[i0*self.d1 as usize + i1]
	    } else {
		&self.out
	    }
    }
}

impl<T> IndexMut<[usize;2]> for A2<T> {
    fn index_mut(&mut self,[i0,i1]:[usize;2])->&mut T {
	if i0 < self.d0 as usize && i1 < self.d1 as usize {
		&mut self.data[i0*self.d1 as usize + i1]
	    } else {
		panic!("Index {},{} out of bounds",i0,i1);
	    }
    }
}
