#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(dead_code)]

mod common;
mod object;
mod tiles;
mod world;
mod room;
mod a2;
mod mini_rng;
mod tile_viewer;
mod ptr;

use common::*;
use a2::A2;
use tiles::{
    Corner,
    Door,
    Tile,
    Random
};
use object::Object;
use world::World;
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
		// renderer:eframe::Renderer::Glow,
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
    world:World,
    tex:Option<TextureHandle>,
    frame_rate:f32,
    play:bool,
    tv:TileViewer,
	message:String,
	door_props_open:bool,
	door_editor:Option<DoorEditor>
}

impl Leved {
	fn new(_cc:&eframe::CreationContext<'_>)->Self {
		let tv = TileViewer::new();
		Self {
			tex:None,
			frame_rate:10.0,
			play:false,
			world:World::new(),
			tv,
			message:String::new(),
			door_props_open:false,
			door_editor:None
		}
	}

	fn message(&mut self,msg:&str) {
		self.message.clear();
		self.message.push_str(msg);
	}
}

const TILE_PALETTE : &'static [(&str,Tile,&str)] = &[
	(" ",Tile::Empty,"EMPTY"),
	("%",Tile::Dirt,"DIRT"),
	("#",Tile::Brick,"BRICK"),
	("~",Tile::Water(Periodic { i:0,m:8,j:0,n:8 }),"WATER"),
	(".",Tile::Grass,"GRASS"),
	("@",Tile::Vortex,"VORTEX"),
	("*",Tile::PyramidStone,"PYRAMID"),
	("W",Tile::Window,"WINDOW"),
	("F",Tile::Fire(Periodic { i:0,m:3,j:0,n:2 }),"FIRE"),
	("q",Tile::MetalRamp(Corner::NW),"METAL NW"),
	("w",Tile::MetalRamp(Corner::NE),"METAL NE"),
	("a",Tile::MetalRamp(Corner::SW),"METAL SW"),
	("s",Tile::MetalRamp(Corner::SE),"METAL SE"),
	("m",Tile::Metal,"METAL"),
	("A",Tile::Alien,"ALIEN"),
	("x",Tile::MetalFoot,"METAL FOOT"),
	("^",Tile::Sky(Random { i:1 }),"SKY"),
	("D",Tile::Door(Door { target:None, key:None, locked:false }),"DOOR"),
	("K",Tile::Object(Object::Key),"KEY"),
	("T",Tile::Object(Object::ToyCar),"TOY CAR"),
	("I",Tile::Object(Object::IceCream),"ICECREAM"),
	("C",Tile::Object(Object::Coin),"COIN"),
	("S",Tile::Object(Object::SquaresAndTriangles),"SQ&TR"),
	("c",Tile::Object(Object::Carrot),"CARROT"),
	("t",Tile::Object(Object::Tomato),"TOMATO"),
	("e",Tile::Object(Object::Eggplant),"EGGPLANT"),
	("b",Tile::Object(Object::Banana),"BANANA"),
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
									let mut room = room_ptr.yank_mut();
									ui.text_edit_singleline(&mut room.name);
								} else {
									ui.label("(No name)");
								}

								ui.with_layout(
									Layout::right_to_left(Align::Center),
									|ui| {
										ui.horizontal(|ui| {
											if ui.button("CONN").clicked() {
											}
											if ui.button("UDW").clicked() {

											}
											if ui.button("ID").clicked() {
												if let Some(room_ptr) = self.tv.room() {
													let room = room_ptr.yank_mut();
													if let Some((iy,ix)) = self.tv.selection1() {
														let map = room.map();
														if let Tile::Door(door) = map[[iy,ix]] {
															self.door_editor =
																Some(DoorEditor {
																	room:room.id,
																	indices:(iy,ix),
																	door
																});
															self.door_props_open = true;
														} else {
															self.message("This is not a door!");
														}
													} else {
														self.message("You have to select a tile by right-clicking");
													}
												} else {
													self.message("There is no room");
												}
											}
										});
									});
							});
							
							ui.separator();

							ScrollArea::both()
								.scroll_bar_visibility(ScrollBarVisibility::AlwaysVisible)
								// .max_width(800.0)
								.max_height(600.0)
								.show(ui,|ui| {
									ui.add(&mut self.tv);
								});

							let tm = self.tv.get_tool_mut();

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
										for &(key,tile,_) in TILE_PALETTE {
											let tool = Tool::Place(tile);
											if key == u {
												*tm = tool;
												break;
											}
										}
									},
									_ => ()
								}
							};

							ui.separator();
							ui.label(&self.message);

							ui.separator();
							let num_rows = 8;
							Grid::new("palette")
								.show(
									ui,
									|ui| {
										let mut j = 0;
										for &(key,tile,name) in TILE_PALETTE {
											let tool = Tool::Place(tile);
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
								}
								if ui.button("LOAD").clicked() {
									let patho = rfd::FileDialog::new().pick_file()
										.map(|pb| pb
											 .into_os_string()
											 .into_string()
											 .unwrap_or_else(|_| "WTF".to_string()));
									if let Some(path) = patho {
										self.world.clear();
										match self.world.load(path) {
											Err(e) => eprintln!("Error: {}",e),
											Ok(()) => {
												if let Some(room) = self.world.rooms.get(&self.world.start_room) {
													self.tv.set_room(Some(Ptr::clone(room)));
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
										  ui.horizontal(|ui| {
											  ui.label("Rooms");
											  ui.with_layout(
												  Layout::right_to_left(Align::Center),
												  |ui| {
													  ui.horizontal(|ui| {
														  if ui.button("ADD").clicked() {
														  }
													  });
												  });
										  });
										  ui.separator();
										  let active_id = self.tv.room().map(|p| p.yank().id);
										  for (&iroom,room_ptr) in self.world.rooms.iter() {
											  let room = room_ptr.yank();
											  ui.horizontal(|ui| {
												  ui.monospace(format!("{:8}",iroom));
												  ui.separator();
												  let active = Some(iroom) == active_id;
												  let rt = RichText::new(&room.name);
												  let rt =
													  if active {
														  rt.strong()
													  } else {
														  rt
													  };
												  if ui.button(rt).clicked() {
													  self.tv.set_room(Some(Ptr::clone(&room_ptr)));
												  }
											  });
										  };
									  });
								  });
					});
				});
		});
	}
}
