use std::collections::VecDeque;

use crate::{
    position::Position,
    facing::Facing,
    world::World,
    tiles::{Target,Door,Tile},
    object::Object,
    sounds::Sounds,
    ptr::*
};

#[derive(Debug)]
pub struct Hero {
    room:usize,
    position:Position,
    carrying:Option<Object>,
    travel_request:Option<(isize,isize)>,
    name:String,
    message:String,
    fat:usize,
    won:bool,
    coins:usize,
    foods:usize,
    sounds:VecDeque<Sounds>
}

const FAT_PENALTY : usize = 256;

impl Hero {
    pub fn position(&self)->Position { self.position }

    pub fn carrying(&self)->Option<Object> { self.carrying }

    pub fn room(&self)->usize { self.room }

    pub fn foods(&self)->usize { self.foods }

    pub fn coins(&self)->usize { self.coins }
    
    pub fn won(&self)->bool { self.won }

    pub fn message(&self)->&str { &self.message }

    pub fn next_sound(&mut self)->Option<Sounds> {
	self.sounds.pop_back()
    }

    pub fn new(world:&World,name:&str)->Self {
	let room = world.start_room;
	let (hi,hj) = world.get_room(room).yank().start;
	Hero{
	    room,
	    position:Position::Block(hi,hj,Facing::Right),
	    carrying:None,
	    travel_request:None,
	    name:name.to_string(),
	    message:String::new(),
	    fat:0,
	    coins:0,
	    foods:0,
	    won:false,
	    sounds:VecDeque::new()
	}
    }

    pub fn say(&mut self,msg:&str) {
	self.message.clear();
	self.message.push_str(msg);
    }

    pub fn carry(&mut self,obj:Object) {
	if obj.is_food() {
	    self.sound(Sounds::EatFood);
	    if obj.is_fattening() {
		self.fat = FAT_PENALTY;
		self.say(&format!("YOU EAT {} AND GET FAT.  WALK TO LOSE WEIGHT",obj.name()));
	    } else {
		self.say(&format!("YOU EAT {}",obj.name()));
	    }
	    self.foods += 1;
	} else if obj == Object::Coin {
	    self.sound(Sounds::PickUpCoin);
	    self.coins += 1;
	} else {
	    self.carrying = Some(obj);
	    self.sound(Sounds::PickUpObject);
	    self.say(&format!("YOU TAKE {}",obj.name()));
	}
    }

    pub fn drop(&mut self,world:&mut World) {
	match self.position {
	    Position::Block(hi,hj,f) =>
		match self.carrying {
		    None => (),
		    Some(obj) => {
			let room_ptr = world.get_room(self.room);
			let mut rm = room_ptr.yank_mut();
			let (di,dj) = f.to_deltas();
			let (hi,hj) = (hi as isize + di,hj as isize + dj);
			if 0 <= hi && hi < rm.rows as isize && 0 <= hj && hj < rm.cols as isize {
			    if rm.map[[hi as usize,hj as usize]] == Tile::Empty {
				rm.map[[hi as usize,hj as usize]] = Tile::Object(obj);
				self.carrying = None;
				self.say(&format!("YOU DROP {}",obj.name()));
			    } else {
				self.say(&format!("YOU CANNOT DROP {} OVER THAT",obj.name()));
			    }
			} else {
			    self.say("YOU CANNOT DROP THINGS OVER THE EDGE");
			}
		    }
		},
	    Position::Walking{ .. } => self.say("YOU CANNOT DROP THINGS WHILE WALKING")
	}
    }

    pub fn pending(&mut self,world:&mut World) {
	match self.travel_request {
	    None => (),
	    Some((di,dj)) => self.travel(world,di,dj)
	}
    }

    pub fn tick(&mut self,world:&mut World) {
	//println!("HERO: {:?}",self);
	self.pending(world);
	match &mut self.position {
	    Position::Block(_,_,_) => self.pending(world),
	    Position::Walking{ from:_,to,ref mut step,total } => {
		if self.fat > 0 {
		    self.fat -= 1;
		}
		*step += 1;
		if *step == *total {
		    self.position = Position::Block(to.0,to.1,self.position.facing());
		    self.pending(world);
		    self.sound(Sounds::Walk);
		}
	    }
	}
    }

    pub fn sound(&mut self,snd:Sounds) {
	self.sounds.push_back(snd);
    }

    pub fn start(&mut self,di:isize,dj:isize) {
	let f2 = Facing::from_deltas(di,dj);
	match &mut self.position {
	    Position::Block(_hi,_hj,f1) => *f1 = f2,
	    Position::Walking{ .. } => ()
	}
	self.travel_request = Some((di,dj))
    }

    pub fn stop(&mut self,di:isize,dj:isize) {
	if self.travel_request == Some((di,dj)) {
	    self.travel_request = None;
	}
    }

    pub fn is_fat(&self)->bool {
	self.fat > 0
    }

    pub fn travel(&mut self,world:&mut World,di:isize,dj:isize) {
	let total =
	    if self.is_fat() {
		32
	    } else {
		8
	    };
	// let hr = self.room;
	match self.position {
	    Position::Walking{ .. } => (),
	    Position::Block(hi0,hj0,f) => {
		let (hi,hj) = (hi0 as isize + di,hj0 as isize + dj);
		let room_ptr = world.get_room(self.room);
		let mut rm = room_ptr.yank_mut();
		if (0 <= hi) & (0 <= hj) {
		    let (hi,hj) = (hi as usize,hj as usize);
		    if (hi < rm.rows) & (hj < rm.cols) {
			match &mut rm.map[[hi,hj]] {
			    Tile::Empty => {
				self.position = Position::Walking{
				    from:(hi0,hj0),
				    to:(hi,hj),
				    step:0,
				    total
				}
			    },
			    Tile::Rainbow => {
				self.won = true;
				self.say(&format!("CONGRATULATIONS {} YOU WON!",self.name));
			    },
			    &mut Tile::Object(o) => {
				let ok =
				    match self.carrying {
					None => true,
					Some(o2) => {
					    if o.is_consumable() {
						true
					    } else {
						self.say(&format!("YOU ARE ALREADY CARRYING {}",o2.name()));
						false
					    }
					}
				    };
				if ok {
				    self.carry(o);
				    rm.map[[hi,hj]] = Tile::Empty;
				    self.position = Position::Walking{
					from:(hi0,hj0),
					to:(hi,hj),
					step:0,
					total
				    }
				}
			    },
			    &mut Tile::Door(Door{ target, key, ref mut locked, .. }) => {
				let ok =
				    if *locked {
					if self.carrying == key {
					    *locked = false;
					    self.carrying = None;
					    match &key {
						None => self.say("YOU OPEN THE DOOR"),
						Some(o) => self.say(&format!("YOU UNLOCK THE DOOR WITH {}",o.name()))
					    };
					    true
					} else {
					    match &key {
						None => self.say("THE DOOR IS FOREVER LOCKED"),
						Some(o) => self.say(&format!("YOU NEED {}",o.name()))
					    };
					    false
					}
				    } else {
					true
				    };
				if ok {
				    match &target {
					None => self.say("THIS DOOR HAS NOT BEEN INSTALLED CORRECTLY"),
					&Some(Target{ room, door }) => {
					    match world.get_room(room).yank().locate_door(door) {
						None => self.say("THIS DOOR LEADS NOWHERE"),
						Some((hi,hj)) => {
						    self.room = room;
						    self.travel_request = None;
						    self.position = Position::Block(hi,hj,f);
						    self.sound(Sounds::GoThroughDoor);
						}
					    }
					}
				    }
				}
			    },
			    _ => () // self.sound(Sounds::BlockedAgainstWall)
			}
		    }
		}
	    }
	}
    }
}
