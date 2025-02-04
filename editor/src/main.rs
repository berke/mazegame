#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(dead_code)]

mod common;
mod refresher;
mod tile_viewer;

pub use mzg_game::*;

use common::*;
use a2::A2;
use tiles::{
    Corner,
    Door,
    Tile,
    Random
};
use object::Object;
use world::{
    World,
    TileAddress
};
use room::Room;
use tile_viewer::{
    TileViewer,
    Tool
};
use tiles::{
    Periodic,
    Target
};
use ptr::*;

fn main()->Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
	viewport:ViewportBuilder::default()
	    .with_maximized(true)
	    .with_inner_size([1440.0,1024.0]),
	multisampling:0,
	centered:true,
	renderer:eframe::Renderer::Glow,
	..Default::default()
    };
    eframe::run_native(
	"Mazegame Level Editor",
	options,
	Box::new(|cc| {
	    egui_extras::install_image_loaders(&cc.egui_ctx);
	    Box::new(Leved::new(cc))
	}),
    )
}

struct DoorEditor {
    room:usize,
    indices:(isize,isize),
    door:Door,
}

struct Leved {
    tex:Option<TextureHandle>,
    frame_rate:f32,
    play:bool,
    tv:TileViewer,
    message:String,
    door_props_open:bool,
    path:Option<PathBuf>,
    door_editor:Option<DoorEditor>,
    delete_safety:bool,
    crop_safety:bool
}

fn using<T,F:FnMut(T)>(x:Option<T>,mut f:F) {
    match x {
	None => (),
	Some(y) => f(y)
    }
}

impl Leved {
    fn new(_cc:&eframe::CreationContext<'_>)->Self {
	let tv = TileViewer::new();
	Self {
	    tex:None,
	    frame_rate:10.0,
	    play:false,
	    tv,
	    message:String::new(),
	    path:None,
	    door_props_open:false,
	    door_editor:None,
	    delete_safety:false,
	    crop_safety:false
	}
    }

    fn message(&mut self,msg:&str) {
	self.message.clear();
	self.message.push_str(msg);
    }


    fn path_so(&self)->Option<String> {
	self.path
	    .as_ref()
	    .and_then(|pb|
		      pb
		      .clone()
		      .into_os_string()
		      .into_string()
		      .ok())
    }

    fn udw(&mut self,_ui:&mut Ui) {
	if let Some((ta1,ta2)) = self.tv.selection1().zip(self.tv.selection2()) {
	    if let Some(tt) = self.tv.world.get_tile(&ta1).zip(self.tv.world.get_tile(&ta2)) {
		match tt {
		    (Tile::Door(mut d),Tile::Object(o)) => {
			d.key = Some(o);
			d.locked = true;
			self.tv.world.set_tile(&ta1,Tile::Door(d));
			return;
		    },
		    _ => ()
		}
	    }
	}
	self.message("Select door in green and object in red");
    }

    fn start(&mut self,_ui:&mut Ui) {
	if let Some(ta) = self.tv.selection1() {
	    self.tv.world.start = Some(ta);
	    return;
	}
	self.message("Select starting position in green");
    }

    fn crop(&mut self,_ui:&mut Ui) {
	if let Some((ta1,ta2)) = self.tv.selection1().zip(self.tv.selection2()) {
	    if ta1.room_id != ta2.room_id {
		self.message("Select red and green corners in the same room");
		return;
	    }
	    let iy0 = ta1.iy.min(ta2.iy);
	    let ix0 = ta1.ix.min(ta2.ix);
	    let iy1 = ta1.iy.max(ta2.iy);
	    let ix1 = ta1.ix.max(ta2.ix);
	    let ny = iy1 - iy0 + 1;
	    let nx = ix1 - ix0 + 1;
	    if nx < 1 || ny < 1 {
		self.message("Room would be empty");
		return;
	    }

	    let room_ptr = self.tv.world.get_room(ta1.room_id);
	    let mut room = room_ptr.yank_mut();
	    room.crop(iy0,ny,ix0,nx);
	    self.message(&format!("Room cropped to {} × {}",ny,nx));
	}
	self.message("Select two corners");
    }
    
    fn connect(&mut self,_ui:&mut Ui) {
	if let Some((ta1,ta2)) = self.tv.selection1().zip(self.tv.selection2()) {
	    if let Some(tt) = self.tv.world.get_tile(&ta1).zip(self.tv.world.get_tile(&ta2)) {
		match tt {
		    (Tile::Door(mut d1 @ Door { target:None, .. }),
		     Tile::Door(mut d2 @ Door { target:None, .. })) => {
			d1.target = Some(Target { room:ta2.room_id,
						  door:d2.id });
			d2.target = Some(Target { room:ta1.room_id,
						  door:d1.id });
			self.tv.world.set_tile(&ta1,Tile::Door(d1));
			self.tv.world.set_tile(&ta2,Tile::Door(d2));
			self.message("Doors connected");
		    },
		    _ => {
			self.message("You need to select two unconnected doors!")
		    }
		}
	    }
	}
    }

    fn save(&mut self,_ui:&mut Ui) {
	if let Some(path) = self.path.as_ref() {
	    match self.tv.world.save(path) {
		Err(e) => self.message(&format!("Error: {}",e)),
		Ok(()) => {
		    self.message(&format!("Saved under {:?}",path));
		}
	    }
	}
    }

    fn play(&mut self,_ui:&mut Ui) {
	use std::process::Command;
	use std::env::Vars;

	if let Some(path) = self.path.as_ref() {
	    match std::env::var("PLAYER_COMMAND") {
		Ok(cmd) => {
		    match Command::new(cmd)
			.arg(path)
			.status() {
			    Ok(status) => {
				if !status.success() {
				    self.message("Player failed");
				}
			    },
			    Err(e) => {
				self.message(&format!("Could not launch player: {}",e));
			    }
			}
		},
		Err(_) => {
		    self.message("No PLAYER_COMMAND environment variable");
		}
	    }
	}
    }
}

const TILE_PALETTE : & [(&str,Tool,&str)] = &[
    ("i",Tool::Nothing,"INFO"),
    (" ",Tool::Place(Tile::Empty),"EMPTY"),
    ("%",Tool::Place(Tile::Dirt),"DIRT"),
    ("#",Tool::Place(Tile::Brick),"BRICK"),
    ("~",Tool::Place(Tile::Water(Periodic { i:0,m:8,j:0,n:8 })),"WATER"),
    (".",Tool::Place(Tile::Grass),"GRASS"),
    ("@",Tool::Place(Tile::Vortex),"VORTEX"),
    ("*",Tool::Place(Tile::PyramidStone),"PYRAMID"),
    ("W",Tool::Place(Tile::Window),"WINDOW"),
    ("F",Tool::Place(Tile::Fire(Periodic { i:0,m:3,j:0,n:2 })),"FIRE"),
    ("q",Tool::Place(Tile::MetalRamp(Corner::NW)),"METAL NW"),
    ("w",Tool::Place(Tile::MetalRamp(Corner::NE)),"METAL NE"),
    ("a",Tool::Place(Tile::MetalRamp(Corner::SW)),"METAL SW"),
    ("s",Tool::Place(Tile::MetalRamp(Corner::SE)),"METAL SE"),
    ("m",Tool::Place(Tile::Metal),"METAL"),
    ("A",Tool::Place(Tile::Alien),"ALIEN"),
    ("x",Tool::Place(Tile::MetalFoot),"METAL FOOT"),
    ("^",Tool::PlaceSky,"SKY"),
    ("D",Tool::Place(Tile::Door(Door { id:0,target:None,key:None,locked:false })),"DOOR"),
    ("K",Tool::Place(Tile::Object(Object::Key)),"KEY"),
    ("T",Tool::Place(Tile::Object(Object::ToyCar)),"TOY CAR"),
    ("I",Tool::Place(Tile::Object(Object::IceCream)),"ICECREAM"),
    ("C",Tool::Place(Tile::Object(Object::Coin)),"COIN"),
    ("S",Tool::Place(Tile::Object(Object::SquaresAndTriangles)),"SQ&TR"),
    ("c",Tool::Place(Tile::Object(Object::Carrot)),"CARROT"),
    ("t",Tool::Place(Tile::Object(Object::Tomato)),"TOMATO"),
    ("e",Tool::Place(Tile::Object(Object::Eggplant)),"EGGPLANT"),
    ("b",Tool::Place(Tile::Object(Object::Banana)),"BANANA"),
    ("R",Tool::Place(Tile::Rainbow),"RAINBOW"),
    ("L",Tool::Lock,"LOCK"),
    ("U",Tool::Unlock,"UNLOCK"),
];

impl eframe::App for Leved {
    fn update(&mut self,ctx:&Context,_frame:&mut eframe::Frame) {
	CentralPanel::default().show(ctx,|ui| {
	    Window::new("Edit door")
		.open(&mut self.door_props_open)
		.vscroll(false)
		.default_width(400.0)
		.default_height(300.0)
		.default_pos(ctx.screen_rect().center())
		.movable(true)
		.show(ctx, |ui| {
		    if let Some(ded) = self.door_editor.as_mut() {
			ui.horizontal(|ui| {
			    let target =
				if let Some(Target { room,door }) = ded.door.target {
				    format!("Room {} door {}",room,door)
				} else {
				    format!("Nowhere!")
				};
			    ui.label(format!("Goes to: {}",target));
			    if ui.button("Change").clicked() {
			    }
			});

			ui.checkbox(&mut ded.door.locked,"Locked");
		    } else {
			ui.label("We are not editing a door right now.");
		    }
		    if ui.button("Change the door").clicked() {
			// if let Some(room_ptr) = self.tv.room() {
			// 	let mut room = room_ptr.yank_mut();
			// 	if let Some((iy,ix)) = self.tv.selection1() {
			// 	}
			// }
		    }
		});
	    StripBuilder::new(ui)
		.size(Size::remainder().at_least(700.0))
		.size(Size::exact(300.0))
		.horizontal(|mut strip| {
		    strip.cell(|ui| {
			ui.vertical(|ui| {
			    ui.horizontal(|ui| {
				if let Some(room_ptr) = self.tv.room() {
				    ui.label("Name:");
				    let mut room = room_ptr.yank_mut();
				    ui.text_edit_singleline(&mut room.name);
				} else {
				    ui.label("No room, create or select one");
				}

				ui.with_layout(
				    Layout::right_to_left(Align::Center),
				    |ui| {
					ui.horizontal(|ui| {
					    if ui.button("CONN").clicked() {
						self.connect(ui);
					    }
					    if ui.button("UDW").clicked() {
						self.udw(ui);
					    }
					    if ui.button("START").clicked() {
						self.start(ui);
					    }
					    if ui.button("CROP").clicked() {
						self.crop(ui);
					    }
					});
				    });
			    });
			    
			    ui.separator();

			    self.tv.ui(ui);
			    if let Some(room_id) = self.tv.take_goto() {
				self.goto_room(room_id);
			    }
			    // ScrollArea::both()
			    // 	.scroll_bar_visibility(ScrollBarVisibility::AlwaysVisible)
			    // // .max_width(800.0)
			    // 	.max_height(600.0)
			    // 	.show(ui,|ui| {
			    // 	    ui.add(&mut self.tv);
			    // 	});

			    let event_filter = EventFilter {
				horizontal_arrows:false,
				vertical_arrows:false,
				tab:false,
				..Default::default()
			    };

			    let events = ui.input(
				|i|
				i.filtered_events(&event_filter));
			    for event in &events {
				match event {
				    Event::Text(u) => {
					match u.as_str() {
					    "u" => self.tv.undo(),
					    "r" => self.tv.redo(),
					    _ => {
						let tm = self.tv.get_tool_mut();

						for &(key,tool,_) in TILE_PALETTE {
						    if key == u {
							*tm = tool;
							break;
						    }
						}
					    }
					}
				    },
				    _ => ()
				}
			    };

			    ui.separator();
			    ui.label(&self.message);

			    let tm = self.tv.get_tool_mut();
			    ui.separator();
			    let num_rows = 8;
			    Grid::new("palette")
				.show(
				    ui,
				    |ui| {
					let mut j = 0;
					for &(key,tool,name) in TILE_PALETTE {
					    ui.selectable_value(
						tm,
						tool,
						format!("{} {}",key,name));
					    j += 1;
					    if j == num_rows {
						ui.end_row();
						j = 0;
					    }
					}
				    });

			    ui.separator();
			    ui.horizontal(|ui| {
				if ui.button("SAVE").clicked() {
				    self.save(ui);
				}
				if ui.button("PLAY").clicked() {
				    self.play(ui);
				}
				if ui.button("SAVE AS").clicked() {
				    let rfd =
					rfd::FileDialog::new()
					.set_title("Save world");

				    let rfd =
					if let Some(path) = self.path.as_ref().and_then(|pb| pb.parent()) {
					    rfd.set_directory(path)
					} else {
					    rfd
					};

				    let rfd =
					if let Some(path_s) = self.path_so() {
					    rfd.set_file_name(path_s)
					} else {
					    rfd
					};

				    if let Some(path) = rfd.save_file() {
					self.path = Some(path);
					self.save(ui);
				    }
				}
				if ui.button("LOAD").clicked() {
				    let rfd = rfd::FileDialog::new()
					.set_title("Load world");

				    let rfd =
					if let Some(path_s) = self.path_so() {
					    rfd.set_file_name(path_s)
					} else {
					    rfd
					};

				    let patho = rfd.pick_file();

				    if let Some(path) = patho {
					self.tv.world.clear();
					match self.tv.world.load(&path) {
					    Err(e) => self.message(&format!("Error: {}",e)),
					    Ok(()) => {
						self.message(&format!("Loaded from {:?}",path));
						self.path = Some(path);
						if let Some(TileAddress { room_id, .. }) = self.tv.world.start {
						    self.goto_room(room_id);
						}
					    }
					}
				    }
				}
			    });
			});
		    });
		    strip.cell(|ui| {
			ScrollArea::vertical()
			    .auto_shrink(false)
			    .show(ui,
				  |ui| {
				      ui.vertical(|ui| {
					  self.room_list(ui);
				      });
				  });
		    });
		});
	});
    }
}

trait ApplyIf {
    fn apply_if<F:FnMut(Self)->Self>(self,x:bool,mut f:F)->Self where Self:Sized {
	if x {
	    f(self)
	} else {
	    self
	}
    }
}

impl ApplyIf for RichText { }

impl Leved {
    fn room_list(&mut self,ui:&mut Ui) {
	ui.horizontal(|ui| {
	    ui.label("Rooms");
	    ui.with_layout(
		Layout::right_to_left(Align::Center),
		|ui| {
		    ui.horizontal(|ui| {
			if ui.button("ADD").clicked() {
			    let id = self.tv.world.last_id().map(|id| id + 1)
				.unwrap_or(0);
			    let room = Room::empty(id,48,48);
			    self.tv.world.insert_room(room);
			    self.goto_room(id);
			}
			let delete_safety = self.delete_safety;
			if delete_safety {
			    if ui.button("CONFIRM DELETE").clicked() {
				if let Some(room_ptr) = self.tv.room() {
				    let room = room_ptr.yank();
				    self.tv.world.delete_room(room.id);
				    self.tv.set_room(None);
				}
				self.delete_safety = false;
			    }
			    if ui.button("CANCEL DELETE").clicked() {
				self.delete_safety = false;
			    }
			} else {
			    if ui.button("DELETE ROOM").clicked() {
				self.delete_safety = true;
			    }
			}
		    });
		});
	});
	ui.separator();
	let active_id = self.tv.room().map(|p| p.yank().id);
	let room_list = self.tv.world.room_list();
	for iroom in room_list {
	    let room_ptr = self.tv.world.get_room(iroom);
	    let room = room_ptr.yank();
	    ui.horizontal(|ui| {
		ui.monospace(format!("{:8}",iroom));
		ui.separator();
		let active = Some(iroom) == active_id;
		let has_sels : Vec<_> = [self.tv.selection1(),self.tv.selection2()]
		    .iter()
		    .map(|selo|
			 selo
			 .map(|sel| sel.room_id == iroom)
			 .unwrap_or(false))
		    .collect();
		ui.monospace(
		    RichText::new(" ")
			.apply_if(has_sels[0],|t| t.background_color(Color32::GREEN)));
		ui.monospace(
		    RichText::new(" ")
			.apply_if(has_sels[1],|t| t.background_color(Color32::RED)));
		let rt = RichText::new(&room.name)
		    .apply_if(active,|t| t.strong());
		if ui.button(rt).clicked() {
		    self.tv.set_room(Some(room_ptr.refer()));
		    self.delete_safety = false;
		}
	    });
	};
    }

    fn goto_room(&mut self,room_id:usize) {
	if let Some(room) = self.tv.world.rooms.get(&room_id) {
	    self.tv.set_room(Some(Ptr::clone(room)));
	}
    }
}
