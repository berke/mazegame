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
use tile_viewer::TileViewer;
use ptr::*;

fn main()->Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
	viewport:ViewportBuilder::default()
	    .with_inner_size([500.0,800.0]),
	multisampling:0,
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

struct Leved {
    world:World,
    tex:Option<TextureHandle>,
    frame_rate:f32,
    play:bool,
    tv:TileViewer
}

impl Leved {
    fn new(_cc:&eframe::CreationContext<'_>)->Self {
	let tv = TileViewer::new();
	Self {
	    tex:None,
	    frame_rate:10.0,
	    play:false,
	    world:World::new(),
	    tv
	}
    }
}

impl eframe::App for Leved {
	fn update(&mut self,ctx:&Context,_frame:&mut eframe::Frame) {
		CentralPanel::default().show(ctx,|ui| {
			StripBuilder::new(ui)
				.size(Size::remainder().at_least(700.0))
				.size(Size::exact(300.0))
				.horizontal(
					|mut strip| {
						strip.cell(|ui| {
							ui.vertical(|ui| {
								if ui.button("Load").clicked() {
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
								ui.add(&mut self.tv);
							});

						});
						strip.cell(|ui| {
							ScrollArea::vertical()
								.auto_shrink(false)
								.show(ui,
									  |ui| {
										  ui.vertical(|ui| {
											  for (iroom,room_ptr) in self.world.rooms.iter() {
												  let room = room_ptr.yank();
												  ui.horizontal(|ui| {
													  ui.monospace(format!("{:8}",iroom));
													  ui.separator();
													  if ui.button(&room.name).clicked() {
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
