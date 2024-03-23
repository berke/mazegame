#![allow(dead_code)]

mod common;
mod facing;
mod hero;
mod position;
mod sounds;
mod synthesizer;

use common::*;

pub fn main() -> Result<(),Box<dyn Error>> {
    println!("Hello........");

    let font_height : usize = 16;
    let font_width : usize = 10; // XXX

    let args : Vec<_> = env::args().collect();
    let font_path : &Path = Path::new(&args[1]);
    let tiles_path = &args[2];
    let world_path = &args[3];
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
    let font = ttf_context.load_font(font_path, font_height as u16)?;
    // font.set_style(sdl2::ttf::FontStyle::BOLD);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(48_000),
        channels: Some(2),
        samples: None
    };

    let device = audio_subsystem.open_queue::<i16, _>(None, &desired_spec)?;
    let spec = device.spec();

    let synth = Synthesizer::new(spec);

    let mut sounds = BTreeMap::new();
    sounds.insert(Sounds::Walk,synth.generate(100.0,100.0,0.025, 1.0,1.0));
    sounds.insert(Sounds::PickUpCoin,synth.generate(1000.0,2000.0,0.100, 0.05,0.25));
    sounds.insert(Sounds::PickUpObject,synth.generate(3000.0,2500.0,0.200, 0.25,0.15));
    sounds.insert(Sounds::EatFood,synth.generate(500.0,400.0,0.200, 0.1,0.3));
    sounds.insert(Sounds::GoThroughDoor,synth.generate(400.0,200.0,0.500, 0.2,1.0));
    sounds.insert(Sounds::BlockedAgainstWall,synth.generate(175.0,150.0,0.100, 0.2,0.2));

    device.resume();

    let (window_width,window_height) = (800,600);
    
    let window = video_subsystem.window("Maze Game",
					window_width as u32,window_height as u32)
	.position_centered()
        // .fullscreen()
	.build()
	.unwrap();
    let (width,height) = window.size();

    let mutil = sdl_context.mouse();
    mutil.show_cursor(false);
    //mutil.set_relative_mouse_mode(true);
    
    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mx = 0;
    let my = font_height + 8;
    // let bw = 32; // Block size
    // let bh = 32; // Block size

    let mut world = World::new();
    world.load(world_path)?;
    let mut hero = Hero::new(&world,"FELIX");
    // let mut walk = Walk::new();

    let clear = |canvas:&mut Canvas<_>|->Result<(),String> {
	canvas.set_draw_color(Color::RGB(  0,  0,  0));
	canvas.fill_rect(Rect::new(0,0,width,height))?;
	Ok(())
    };

    struct Redrawer<'a> {
	mx:usize,
	my:usize,
	bw:usize,
	bh:usize,
	ox:usize,
	oy:usize,
	rainbow:Vec<sdl2::pixels::Color>,
	rainbow_index:usize,
	texture:Texture<'a>,
	rng:MiniRNG
    }

    const BW : usize = 16;
    const BH : usize = 16;

    impl<'a> Redrawer<'a> {
	fn new<T>(mx:usize,my:usize,ox:usize,oy:usize,tiles:&str,texture_creator:&'a TextureCreator<T>)->Self {
	    let tiles = Surface::from_file(tiles).unwrap();
	    let texture = texture_creator.create_texture_from_surface(&tiles).unwrap();
	    let rainbow = vec![
		Color::RGB(255,  0,  0),
		Color::RGB(255,255,  0),
		Color::RGB(  0,255,  0),
		Color::RGB(  0,255,255),
		Color::RGB(  0,  0,255),
		Color::RGB(255,  0,255),
	    ];
	    Redrawer{
		mx,
		my,
		bw:BW,
		bh:BH,
		ox,
		oy,
		rainbow_index:0,
		rainbow,
		texture,
		rng:MiniRNG::new(1234)
	    }
	}

	fn random_color(&mut self)->sdl2::pixels::Color {
	    let c = self.rng.next();
	    let r = c & 255;
	    let g = (c >> 8) & 255;
	    let b = (c >> 16) & 255;
	    Color::RGB(r as u8,g as u8,b as u8)
	}

	fn redraw<T:RenderTarget>(&mut self,canvas:&mut Canvas<T>,rm:&Room,hero:&Hero)->Result<(),String> {
	    self.rainbow_index += 1;
	    if self.rainbow_index == self.rainbow.len() {
		self.rainbow_index = 0;
	    }
	    let canvas = Mutex::new(canvas);
	    let draw = |x:usize,y:usize,tl| {
		let dst = Rect::new(x as i32,
				    y as i32,
				    self.bw as u32,
				    self.bh as u32);
		let fill = |col| {
		    let mut canvas = canvas.lock().unwrap();
		    canvas.set_draw_color(col);
		    canvas.fill_rect(dst).unwrap();
		};
		let tile = |u,v| {
		    let mut canvas = canvas.lock().unwrap();
		    let (tw,th) = (16,16);
		    let src = Rect::new((v*tw) as i32,
					(u*th) as i32,
					tw as u32,
					th as u32);
		    canvas.copy(&self.texture,Some(src),Some(dst)).unwrap();
		};
		fill(Color::RGB(255,255,0));
		match tl {
		    Tile::Fire(p) => tile(4+p.i,4),
		    Tile::Brick => tile(0,0),
		    Tile::Metal => tile(0,12),
		    Tile::Alien => tile(0,14),
		    Tile::Sky(Random{ i }) => tile(i as u16,10),
		    Tile::MetalRamp(Corner::NW) => tile(2,13),
		    Tile::MetalRamp(Corner::NE) => tile(2,12),
		    Tile::MetalRamp(Corner::SW) => tile(1,12),
		    Tile::MetalRamp(Corner::SE) => tile(1,13),
		    Tile::MetalFoot => tile(3,12),
		    Tile::Object(o) => match o {
			Object::Coin => tile(0,1),
			Object::IceCream => tile(0,3),
			Object::Key => tile(0,4),
			Object::ToyCar => tile(0,5),
			Object::SquaresAndTriangles => tile(0,6),
			Object::Tomato => tile(1,5),
			Object::Eggplant => tile(1,6),
			Object::Banana => tile(1,7),
			Object::Carrot => tile(1,8)
		    },
		    Tile::Door(Door{ key:None, .. }) => tile(0,2),
		    Tile::Door(Door{ key:Some(_), locked:true, ..}) => tile(1,3),
		    Tile::Door(Door{ key:Some(_), locked:false, ..}) => tile(1,4),
		    Tile::Vortex => tile(0,7),
		    Tile::Grass => tile(0,8),
		    Tile::Dirt => tile(2,8),
		    Tile::PyramidStone => tile(3,8),
		    Tile::Window => tile(0,11),
		    Tile::Water(p) => tile(p.i,9),
		    Tile::Empty => fill(Color::RGB(  0,  0,  0)),
		    Tile::Rainbow => fill(self.rainbow[self.rainbow_index]),
		}
	    };
	    
	    for i in 0..rm.rows {
		for j in 0..rm.cols {
		    draw(self.mx + j*self.bw,
			 self.my + i*self.bh,
			 rm.map[[i,j]]);
		}
	    }

	    {
		match &hero.carrying() {
		    None => (),
		    Some(o) => draw(self.ox,self.oy,Tile::Object(*o))
		}
	    }

	    {
		let u = match hero.position().facing() {
		    Facing::Right => 1,
		    Facing::Left => 2,
		    Facing::Up => 3,
		    Facing::Down => 4
		};
		let u = if hero.is_fat() { u + 5 } else { u };
		let (hx,hy,v) =
		    match hero.position() {
			Position::Block(hi,hj,_) => (self.mx + hj*self.bw, self.my + hi*self.bh,0),
			Position::Walking{ from:(hi0,hj0),to:(hi1,hj1),step,total } =>
			    ((self.mx as isize +
			      (self.bw*hj0) as isize +
			      (self.bw*step) as isize*(hj1 as isize-hj0 as isize)/total as isize) as usize,
			     (self.my as isize +
			      (self.bh*hi0) as isize +
			      (self.bh*step) as isize*(hi1 as isize-hi0 as isize)/total as isize) as usize,
			     1+step%2)
		    };
		let dst = Rect::new(hx as i32,
				    hy as i32,
				    self.bw as u32,
				    self.bh as u32);
		let mut canvas = canvas.lock().unwrap();
		let (tw,th) = (16,16);
		let src = Rect::new((v*tw) as i32,
				    (u*th) as i32,
				    tw as u32,
				    th as u32);
		canvas.copy(&self.texture,Some(src),Some(dst)).unwrap();
	    }

	    if hero.won() {
		let mut canvas = canvas.lock().unwrap();
		for _ in 0..500 {
		    let x0 = self.mx as i32 + self.rng.sample_u32((self.bw*rm.cols) as u32) as i32;
		    let x1 = self.mx as i32 + self.rng.sample_u32((self.bw*rm.cols) as u32) as i32;
		    let y0 = self.my as i32 + self.rng.sample_u32((self.bh*rm.rows) as u32) as i32;
		    let y1 = self.my as i32 + self.rng.sample_u32((self.bh*rm.rows) as u32) as i32;
		    canvas.set_draw_color(self.random_color());
		    canvas.draw_line(Point::new(x0,y0),Point::new(x1,y1))?;
		}
	    }
	    Ok(())
	}
    }

    let mut redrawer = Redrawer::new(mx,my,width as usize-BW,height as usize-font_height-8,
				     tiles_path,&texture_creator);

    let write = |canvas:&mut Canvas<_>,x,y,text:&str,color| {
	if !text.is_empty() {
	    let surface = font.render(text).blended(color).unwrap();
	    let texture = texture_creator.create_texture_from_surface(&surface).unwrap();
	    let TextureQuery { width, height, .. } = texture.query();
	    let target = Rect::new(x as i32,y as i32,width,height);
	    canvas.copy(&texture, None, Some(target)).unwrap();
	    width
	} else {
	    0
	}
    };

    // println!("WORLD: {:?}",world);
    'running: loop {
	clear(&mut canvas)?;
	if hero.won() {
	    write(&mut canvas,0,0,"YOU WON THE GAME !!!",redrawer.random_color());
	} else {
	    write(&mut canvas,0,0,
		  &world.get_room(hero.room()).yank().name,Color::RGB(255,  0,  0));
	}
	write(&mut canvas,0,height-font_height as u32-8,
	      hero.message(),
	      Color::RGB(255,255,255));
	write(&mut canvas,width-font_width as u32*24,height-font_height as u32-8,
	      &format!("FOODS {:5}",hero.foods()),
	      Color::RGB(  0,255,0));
	write(&mut canvas,width-font_width as u32*12,height-font_height as u32-8,
	      &format!("COINS {:5}",hero.coins()),
	      Color::RGB(255,255,0));

	loop {
	    match hero.next_sound() {
		None => break,
		Some(snd) =>
		    match sounds.get(&snd) {
			None => (), // println!("Sound {:?} not found",snd),
			Some(w) => {
			    let _ = device.queue(w); // XXX ignore?
			}
		    }
	    }
	}

	for event in event_pump.poll_iter() {
	    match event {
		Event::Quit {..} => break 'running,
		// Event::MouseWheel { direction: MouseWheelDirection::Normal, .. } |
		// Event::MouseButtonUp { mouse_btn:MouseButton::X1, .. } => {
		//     star_scale=star_scale+0.01
		// },
		// Event::MouseWheel { direction: MouseWheelDirection::Flipped, .. } |
		// Event::MouseButtonUp { mouse_btn:MouseButton::X2, .. } => {
		//     star_scale=star_scale-0.01
		// },
		// Event::MouseMotion { xrel, .. } => {
		//     move_pointer(0.01*(xrel as f64));
		// },
		Event::KeyUp { keycode: Some(kc), .. } => {
		    // let kc_i32 = keycode_to_i32(kc);
		    match kc {
			Keycode::Left => hero.stop(0,-1),
			Keycode::Right => hero.stop(0,1), 
			Keycode::Up  => hero.stop(-1,0),
			Keycode::Down  => hero.stop(1,0),
			_ => (),
		    }
		},
		Event::KeyDown { keycode: Some(kc), repeat, .. } => {
		    match kc {
			Keycode::Escape => {
			    println!("BYE");
			    break 'running
			},
			Keycode::F5 => {
			    println!("Reloading...");
			    world.clear();
			    world.load(world_path)?;
			},
			_ => {
			    if !repeat {
				match kc {
				    Keycode::Left => {
					hero.start(0,-1)
				    },
				    Keycode::Right => hero.start(0,1),
				    Keycode::Up => hero.start(-1,0),
				    Keycode::Down => hero.start(1,0),
				    Keycode::Space => hero.drop(&mut world),
				    _ => (),
				}
			    }
			}
		    }
		},
		// Event::TextInput { text:t, .. } => {
		//     if text.len() < max_column {
		//         text.push_str(&t.to_ascii_uppercase())
		//     }
		// },
		_ => {}
	    }
	}

	hero.tick(&mut world);


	{
	    let room_ptr = world.get_room(hero.room());
	    let mut room = room_ptr.yank_mut();
	    room.next();
	    redrawer.redraw(&mut canvas,
			    &room,
			    &hero)?;
	}
	canvas.present();
	::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
    Ok(())
}
