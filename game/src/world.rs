use anyhow::{
    anyhow,
    bail,
    Result
};

use std::{
    fs::File,
    path::Path,
    cell::RefCell,
    io::{
	BufReader,
	BufRead,
	BufWriter,
	Seek
    },
    collections::{
	BTreeMap
    }
};

use serde::{
    Deserialize,
    Serialize
};

use crate::{
    room::Room,
    object::Object,
    tiles::*,
    ptr::*
};

#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct World {
    pub rooms:BTreeMap<usize,Ptr<Room>>,
    pub start:Option<TileAddress>,
}

#[derive(Copy,Clone,Debug,PartialEq,Serialize,Deserialize)]
pub struct TileAddress {
    pub room_id:usize,
    pub iy:usize,
    pub ix:usize
}

impl World {
    pub fn clear(&mut self) {
	self.rooms.clear();
	self.start = None
    }

    pub fn save<P:AsRef<Path>>(&self,path:P)->Result<()> {
	let fd = File::create(path)?;
	let mut buf = BufWriter::new(fd);
	ron::ser::to_writer(&mut buf,self)?;
	Ok(())
    }

    pub fn room_list(&self)->Vec<usize> {
	self.rooms.keys().copied().collect()
    }

    pub fn load<P:AsRef<Path>>(&mut self,path:P)->Result<()> {
	let fd = File::open(path)?;
	let mut buf = BufReader::new(fd);
	if let Ok(this) = ron::de::from_reader::<_,World>(&mut buf) {
	    self.rooms = this.rooms;
	    self.start = this.start;
	    Ok(())
	} else {
	    buf.rewind()?;
	    let line_number = RefCell::new(0);
	    let mut f = ||->Result<String> {
		let mut u = String::new();
		let _ = buf.read_line(&mut u)?;
		let mut ln = line_number.borrow_mut();
		*ln += 1;
		Ok(u.trim_end_matches('\n').to_string())
	    };
	    let g = |u:&str|->Result<usize> {
		u.parse::<usize>()
		    .map_err(|_| anyhow!("Bad integer {u} at line {}",
					 *(line_number.borrow())))
	    };
	    loop {
		let line = f()?;
		if line.trim_start().starts_with("//") || line.trim_start().is_empty() {
		    continue;
		}
		let words : Vec<&str> = line.split(' ').collect();
		match words[..] {
		    ["END"] => break,
		    ["CONN",room1,door1,room2,door2] =>
			self.connect(g(room1)?,g(door1)?,g(room2)?,g(door2)?),
		    ["LOCK",room,door,object] => {
			let cs : Vec<char> = object.chars().collect();
			let obj = 
			    if cs.len() == 1 {
				Object::from_char(cs[0])?
			    } else {
				bail!("Invalid object string {:?}",object);
			    };
			self.lock_door_with(g(room)?,g(door)?,obj);
		    },
		    ["START",room] => {
			let _ = g(room)?;
		    },
		    ["ROOM",id] => {
			let name = f()?;
			let mut descr : Vec<String> = Vec::new();
			loop {
			    let line = f()?;
			    if line.starts_with(' ') {
				let (_,rest) = line.split_once(' ').unwrap();
				descr.push(rest.to_string());
			    } else if line == "ENDROOM" {
				break;
			    } else {
				bail!("Invalid room line");
			    }
			}
			let descr_ref : Vec<&str> = descr.iter().map(|x| x.as_str())
			    .collect();
			let start = self.add_room(g(id)?,&name,&descr_ref[..]);
			self.start = self.start.or(start);
		    },
		    _ => bail!("Invalid stanza {:?}",line)
		};
	    }
	    Ok(())
	}
    }

    pub fn new()->Self {
	World {
	    rooms:BTreeMap::new(),
	    start:None,
	}
    }

    pub fn last_id(&self)->Option<usize> {
	self.rooms.last_key_value().map(|(&k,_v)| k)
    }

    pub fn insert_room(&mut self,room:Room) {
	self.rooms.insert(room.id,Ptr::make(room));
    }

    pub fn add_room(&mut self,id:usize,name:&str,descr:&[&str])->Option<TileAddress> {
	let (room,start) = Room::new(id,name,descr);
	self.insert_room(room);
	start
    }

    pub fn delete_room(&mut self,id:usize) {
	self.rooms.remove(&id);
    }

    pub fn get_room(&self,id:usize)->Ptr<Room> {
	self.rooms.get(&id).expect(&format!("Room {} not found",id)).refer()
    }

    pub fn lock_door_with(&mut self,room:usize,door:usize,obj:Object) {
	let mut room = self.rooms.get(&room).unwrap().yank_mut();
	let door = room.find_door(door);
	door.locked = true;
	door.key = Some(obj);
    }

    pub fn connect(&mut self,room1:usize,door1:usize,room2:usize,door2:usize) {
	{
	    let mut r1 = self.rooms.get(&room1).unwrap().yank_mut();
	    let d1 = r1.find_door(door1);
	    if d1.target.is_some() {
		panic!("Cannot connect {},{} to {},{} -- origin in use by {:?}",room1,door1,room2,door2,d1.target)
	    }
	    d1.target = Some(Target{ room:room2, door:door2 });
	}
	{
	    let mut r2 = self.rooms.get(&room2).unwrap().yank_mut();
	    let d2 = r2.find_door(door2);
	    if d2.target.is_some() {
		panic!("Cannot connect {},{} to {},{} -- destination in use by {:?}",room1,door1,room2,door2,d2.target)
	    }
	    d2.target = Some(Target{ room:room1, door:door1 });
	}
    }

    pub fn get_tile(&self,ta:&TileAddress)->Option<Tile> {
	self.rooms.get(&ta.room_id)
	    .map(|room_ptr| {
		let room = room_ptr.yank();
		room.map[[ta.iy,ta.ix]]
	    })
    }

    pub fn set_tile(&self,ta:&TileAddress,tile:Tile) {
	self.rooms.get(&ta.room_id)
	    .map_or((),|room_ptr| {
		let mut room = room_ptr.yank_mut();
		let _ = room.modify(ta.iy,ta.ix,tile);
	    })
    }
}

