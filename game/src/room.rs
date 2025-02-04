use std::collections::BTreeMap;

use serde::{
    Deserialize,
    Serialize
};

use crate::{
    mini_rng::MiniRNG,
    object::Object,
    a2::A2,
    tiles::*,
    world::TileAddress
};

#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct Room {
    pub id:usize,
    pub rows:usize,
    pub cols:usize,
    pub map:A2<Tile>,
    pub doors:BTreeMap<usize,(usize,usize)>,
    pub name:String,
}

impl Room {
    pub fn crop(&mut self,iy0:usize,ny:usize,ix0:usize,nx:usize) {
	self.rows = ny;
	self.cols = nx;
	let mut map = A2::new((ny as isize,nx as isize),Tile::Empty);
	for iy in 0..ny {
	    for ix in 0..nx {
		map[[iy,ix]] = self.map[[iy0 + iy,ix0 + ix]];
	    }
	}
	self.map = map;
	self.reindex_doors();
    }

    fn reindex_doors(&mut self) {
	self.doors.clear();
	for iy in 0..self.rows {
	    for ix in 0..self.cols {
		if let Tile::Door(d) = self.map[[iy,ix]] {
		    self.doors.insert(d.id,(iy,ix));
		}
	    }
	}
    }

    pub fn map(&self)->&A2<Tile> {
	&self.map
    }

    pub fn map_mut(&mut self)->&mut A2<Tile> {
	&mut self.map
    }

    pub fn modify(&mut self,iy:usize,ix:usize,mut tile:Tile)->Tile {
	use Tile::*;

	let old_tile = self.map[[iy,ix]];

	if let Door(d_old) = old_tile {
	    self.doors.remove(&d_old.id);
	}
	
	if let Door(d) = &mut tile {
	    if self.doors.contains_key(&d.id) {
		d.id = self.doors.last_key_value().map(|(&k,_v)| k + 1)
		    .unwrap_or(0);
		// println!("Adjusted door ID to {}",d.id);
	    }

	    self.doors.insert(d.id,(iy,ix));
	}
	
	// println!("Setting ({},{}) to {:?}",iy,ix,tile);
	self.map[[iy,ix]] = tile;

	tile
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

    pub fn locate_door(&self,door:usize)->Option<(usize,usize)> {
	self.doors.get(&door).copied()
    }
    
    pub fn find_door(&mut self,door:usize)->&mut Door {
	match self.doors.get(&door) {
	    None => panic!("Door not defined"),
	    Some(&(i,j)) => {
		match &mut self.map[[i,j]] {
		    Tile::Door(d) => d,
		    _ => panic!("Not a door")
		}
	    }
	}
    }

    pub fn empty(id:usize,rows:usize,cols:usize)->Self {
	let map = A2::new((rows as isize,cols as isize),Tile::Empty);
	Self {
	    id,
	    rows,
	    cols,
	    map,
	    doors:BTreeMap::new(),
	    name:format!("Room {}",id)
	}
    }

    pub fn new(id:usize,name:&str,a:&[&str])->(Self,Option<TileAddress>) {
	let mut rng = MiniRNG::new(1);
	// let a : Vec<&str> = descr.split('\n').collect();
	let rows = a.len();
	let cols = a[0].len();
	let mut map = A2::new((rows as isize,cols as isize),Tile::Empty);
	let mut doors = BTreeMap::new();
	let mut start = None;
	for i in 0..rows {
	    // println!("ROW {:2} [{}]",i,a[i]);
	    for (j,c) in a[i].chars().enumerate() {
		let t =
		    match c {
			' ' => Tile::Empty,
			'H' => {
			    start = Some(TileAddress { room_id:id,
						       iy:i,ix:j });
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
			    doors.insert(x,(i,j));
			    Tile::Door(Door{ id:x,target:None,key:None,
					     locked:false })
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
	(Self {
	    id,
	    rows,
	    cols,
	    map,
	    doors,
	    name:name.to_string()
	},
	 start)
    }
}
