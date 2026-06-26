use crate::v2d::V2D;

const GRAVITY_TUNE: f64 = 0.25;
const SPRINGING_TUNE: f64 = 0.25;
const WAVESPEED_TUNE: f64 = 2.0;

pub enum WaveDirection {
    Forward,
    Reverse,
    Manual,
}

#[derive(PartialEq, Eq)]
pub enum GravityDirection {
    Down,
    Up,
    Off,
}

#[derive(PartialEq, Eq)]
pub enum WallHit {
    Untouched,
    Left,
    Right
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum SurfaceSticky {
    #[default]
    Sticky,
    Slippy,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct World {
    pub gravity: f64,
    pub friction: f64,
    pub springyness: f64,
    pub surface_friction: f64,
    pub surface_reflection: f64,
    pub stickyness: SurfaceSticky,
    pub width: f64,
    pub height: f64,

    pub(crate) gravity_direction: f64,
}

impl World {
    pub fn new(width: f64, height: f64, gravity: f64, friction: f64, springyness: f64,
               surface_reflection: f64, surface_friction: f64, gravity_direction: GravityDirection) -> Self {
        Self {
            width, height,
            gravity, friction, springyness, surface_reflection, surface_friction,
            gravity_direction: match gravity_direction {
                GravityDirection::Down => -1.0,
                GravityDirection::Up => 1.0,
                GravityDirection::Off => 0.0,
            },
            stickyness: if surface_friction == 1.0 { SurfaceSticky::Slippy } else { SurfaceSticky::Sticky },
        }
    }
    pub fn set_stickyness(&mut self, stickyness: SurfaceSticky) {
        match stickyness {
            SurfaceSticky::Sticky => {
                self.surface_friction = 0.1;
                self.stickyness = SurfaceSticky::Sticky;
            },
            SurfaceSticky::Slippy => {
                self.surface_friction = 1.0;
                self.stickyness = SurfaceSticky::Slippy;
            }
        }
    }
    pub fn set_gravity_dir(&mut self, direction: GravityDirection) {
        match direction {
            GravityDirection::Down => self.gravity_direction = -1.0,
            GravityDirection::Off => self.gravity_direction = 0.0,
            GravityDirection::Up => self.gravity_direction = 1.0,
        }
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Wave {
    pub amplitude: f64,
    pub speed: f64,
    pub angle: f64,
    pub autoreverse: bool,
    pub(crate) direction: f64,
}

impl Wave {
    pub fn new(amplitude: f64, speed: f64, angle: f64, autoreverse: bool, direction: WaveDirection) -> Self {
        Self {
            amplitude, speed, angle, autoreverse,
            direction: match direction {
                WaveDirection::Forward => -1.0,
                WaveDirection::Reverse => 1.0,
                WaveDirection::Manual => 0.0,
            }
        }
    }
    pub fn set_direction(&mut self, direction: WaveDirection) {
        self.direction = match direction {
            WaveDirection::Forward => -1.0,
            WaveDirection::Reverse => 1.0,
            WaveDirection::Manual => 0.0,
        }
    }
    pub fn step(&mut self) {
        self.angle += self.speed * self.direction * WAVESPEED_TUNE;
    }
    pub fn output(&self, sense: f64, phase: f64) -> f64 {
        1.0 + sense * self.amplitude * (self.angle + phase).sin()
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Mass {
    pub pos: V2D,
    pub prv: V2D,
    pub vel: V2D,
    pub(crate) force: V2D,

    pub fixed: bool,
}

impl Mass {
    pub fn new(pos: V2D, vel: V2D, fixed: bool) -> Self {
        Self {
            pos, vel, fixed,
            prv: pos,
            force: V2D::null(),
        }
    }
    pub fn approx_pos(&self, alpha: f64) -> V2D {
        self.pos * alpha + self.prv * (1.0 - alpha)
    }
}


#[derive(Clone, Copy, Default, Debug)]
pub struct Spring {
    pub restlength: f64,
    pub a: usize,
    pub b: usize
}

impl Spring {
    pub fn new(a: usize, b: usize, restlength: f64) -> Self {
        Self {
            a, b, restlength
        }
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Muscle {
    pub restlength: f64,
    pub phase: f64,
    pub sense: f64,
    pub spring_idx: usize,
}

impl Muscle {
    pub fn acted_length(&self, wave: Wave) -> f64 {
        self.restlength * wave.output(self.sense, self.phase)
    }
}

pub struct Model {
    last_wall_hit: WallHit,
    masses: Vec<Mass>,
    springs: Vec<Spring>,
    muscles: Vec<Muscle>,
}

impl Model {
    pub fn new() -> Self {
        Self {
            last_wall_hit: WallHit::Untouched,
            masses: Vec::new(),
            springs: Vec::new(),
            muscles: Vec::new(),
        }
    }
    pub fn get_masses(&self) -> &Vec<Mass> {
        &self.masses
    }

    pub fn get_springs(&self) -> &Vec<Spring> {
        &self.springs
    }

    pub fn get_muscles(&self) -> &Vec<Muscle> {
        &self.muscles
    }

    pub fn get_mass(&self, idx: usize) -> Mass {
        self.masses[idx]
    }
    pub fn get_spring(&self, idx: usize) -> Spring {
        self.springs[idx]
    }


    pub fn add_mass(&mut self, mass: Mass) -> usize {
        self.masses.push(mass);
        self.masses.len()
    }

    pub fn add_spring(&mut self, spring: Spring) -> usize {
        self.springs.push(spring);
        self.springs.len()
    }

    pub fn attach_muscle(&mut self, spr_idx: usize, sense: f64, phase: f64) -> usize {
        let muscle = Muscle {
            restlength: self.springs[spr_idx].restlength,
            sense, phase,
            spring_idx: spr_idx,
        };
        self.muscles.push(muscle);
        self.muscles.len()
    }

    fn clear_forces(&mut self) {
        for mass in &mut self.masses {
            mass.force = V2D::null();
        }
    }

    fn springing(&mut self, springyness: f64) {
        for spring in &mut self.springs {
            let a = &self.masses[spring.a];
            let b = &self.masses[spring.b];
            let ab = b.pos - a.pos;
            let l = ab.mag();
            let f_spr = (ab / l) * (l - spring.restlength) * springyness * SPRINGING_TUNE;

            self.masses[spring.a].force += f_spr;
            self.masses[spring.b].force -= f_spr;
        }
    }
    fn env_affect(&mut self, world: World) {
        for mass in &mut self.masses {
            mass.force.y += world.gravity * world.gravity_direction * GRAVITY_TUNE;
        }
    }
    fn wave_step(&mut self, wave: &mut Wave) {
        for muscle in &mut self.muscles {
            self.springs[muscle.spring_idx].restlength = muscle.acted_length(*wave);
        }

        wave.step();
    }
    fn capture_last(&mut self) {
        for mass in &mut self.masses {
            mass.prv = mass.pos;
        }
    }

    pub fn step(&mut self, wave: &mut Wave, world: World) {
        self.capture_last();
        self.clear_forces();
        self.springing(world.springyness);
        self.env_affect(world);


        // Euler integrator.
        for mass in &mut self.masses {
            if mass.fixed { continue; }

            // force acceleration.
            mass.vel += mass.force;

            // Boundary corrections and surface collisions.
            let detect_tol = 1e-6;
            let vel_tol = 1e-6;
            // Left wall.
            if mass.pos.x < 0.0 + detect_tol && mass.vel.x < 0.0 {
                mass.pos.x = 0.0;
                mass.vel.x *= -world.surface_reflection;
                mass.vel.y *= world.surface_friction;
                if wave.autoreverse && self.last_wall_hit != WallHit::Left {
                    wave.direction *= -1.0;
                    self.last_wall_hit = WallHit::Left;
                }
            // Right wall.
            } else if mass.pos.x > world.width - detect_tol && mass.vel.x > 0.0 {
                mass.pos.x = world.width;
                mass.vel.x *= -world.surface_reflection;
                mass.vel.y *= world.surface_friction;
                if wave.autoreverse && self.last_wall_hit != WallHit::Right {
                    wave.direction *= -1.0;
                    self.last_wall_hit = WallHit::Right;
                }
            }

            // Floor.
            if mass.pos.y < 0.0 + detect_tol && mass.vel.y < 0.0 + vel_tol && mass.vel.y < 0.0 {
                mass.pos.y = 0.0;
                mass.vel.y *= -world.surface_reflection;
                mass.vel.x *= world.surface_friction;
            // Ceiling.
            } else if mass.pos.y > world.height - detect_tol && mass.vel.y > 0.0 {
                mass.pos.y = world.height;
                mass.vel.y *= -world.surface_reflection;
                mass.vel.x *= world.surface_friction;
            }

            // Environment friction.
            mass.vel *= 1.0 - world.friction;

            // Inertia velocity.
            mass.pos += mass.vel;

        }

        self.wave_step(wave);
    }
}


