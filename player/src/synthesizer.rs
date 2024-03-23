use sdl2::audio::AudioSpec;

pub struct Synthesizer {
    channels:usize,
    f_sample:f32
}

impl Synthesizer {
    pub fn new(spec:&AudioSpec)->Self {
	Synthesizer{ channels:spec.channels as usize,
		     f_sample:spec.freq as f32 }
    }

    pub fn generate(&self,f1:f32,f2:f32,dur:f32,vol1:f32,vol2:f32)->Vec<i16> {
	let m = (dur * self.f_sample) as usize;
	let mut u = Vec::new();
	let t_attack = 0.2*dur;
	let t_decay = 0.2*dur;
	u.resize(self.channels as usize * m,0);
	{
	    let mut p = u.as_mut_slice();
	    for i in 0..m as isize {
		let t = (i-1) as f32/self.f_sample;
		let f = f1+(f2-f1)*t/dur;
		let vol = (vol1.ln()+(vol2.ln()-vol1.ln())*t/dur).exp();
		let a = ((5.0*(t/t_attack - 1.0)).tanh() + 1.0)/2.0;
		let b = ((5.0*((dur-t)/t_decay - 1.0)).tanh() + 1.0)/2.0;
		let x = vol*a*b*(2.0*std::f32::consts::PI*f*t).cos();
		let y = (32767.0*x) as i16;
		for j in 0..self.channels {
		    p[j] = y;
		}
		p = &mut p[self.channels..];
	    }
	}
	u
    }
}
