use crate::{
    mini_rng::MiniRNG,
    object::Object,
    a2::A2,
    tiles::*
};

#[derive(Debug)]
pub struct Room {
    pub id:usize,
    pub rows:usize,
    pub cols:usize,
    pub map:A2<Tile>,
    pub doors:Vec<Option<(usize,usize)>>,
    pub name:String,
    pub start:(usize,usize)
}

impl Room {
    pub fn map(&self)->&A2<Tile> {
	&self.map
    }

    pub fn map_mut(&mut self)->&mut A2<Tile> {
	&mut self.map
    }
    
    pub fn dims(&self)->(usize,usize) {
	(self.rows,self.cols)
    }

    pub fn next(&mut self) {
	for i in 0..self.rows {
	    for j in 0..self.cols {
		self.map[[i,j]].next();
	    }
	}
    }

    pub fn find_door(&mut self,door:usize)->&mut Door {
	match self.doors[door] {
	    None => panic!("Door not defined"),
	    Some((i,j)) => {
		match &mut self.map[[i,j]] {
		    Tile::Door(d) => d,
		    _ => panic!("Not a door")
		}
	    }
	}
    }

    pub fn new(id:usize,name:&str,a:&[&str],rng:&mut MiniRNG)->Self {
	// let a : Vec<&str> = descr.split('\n').collect();
	let rows = a.len();
	let cols = a[0].len();
	let mut map = A2::new((rows as isize,cols as isize),Tile::Empty);
	let mut doors = Vec::new();
	let mut start = (0,0);
	doors.resize(10,None);
	for i in 0..rows {
	    // println!("ROW {:2} [{}]",i,a[i]);
	    for (j,c) in a[i].chars().enumerate() {
		let t =
		    match c {
			' ' => Tile::Empty,
			'H' => {
			    start = (i,j);
			    Tile::Empty
			},
			'#' => Tile::Brick,
			'R' => Tile::Rainbow,
			'~' => Tile::Water(Periodic::new(8,8)),
			'@' => Tile::Vortex,
			'.' => Tile::Grass,
			'%' => Tile::Dirt,
			'*' => Tile::PyramidStone,
			'W' => Tile::Window,
			'F' => Tile::Fire(Periodic::new(3,2)),
			'q' => Tile::MetalRamp(Corner::NW),
			'w' => Tile::MetalRamp(Corner::NE),
			'a' => Tile::MetalRamp(Corner::SW),
			's' => Tile::MetalRamp(Corner::SE),
			'm' => Tile::Metal,
			'A' => Tile::Alien,
			'x' => Tile::MetalFoot,
			'^' => Tile::Sky(Random::new(rng.sample_u32(20))),
			'0'..='9' => {
			    let x = c.to_digit(10).unwrap() as usize;
			    doors[x] = Some((i,j));
			    Tile::Door(Door{ target:None, key:None, locked:false })
			},
			_ => if let Ok(obj) = Object::from_char(c) {
			    Tile::Object(obj)
			} else {
			    panic!("Unsupported tile {}",c)
			}
		    };
		map[[i,j]] = t;
	    }
	}
	Self {
	    id,
	    rows,
	    cols,
	    map,
	    doors,
	    start,
	    name:name.to_string()
	}
    }
}
