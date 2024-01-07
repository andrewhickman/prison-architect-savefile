use noise::NoiseFn;
use prison_architect_savefile::Node;
use rand::rngs::SmallRng;
use rand::RngCore;
use rand::SeedableRng;

const WIDTH: u32 = 300;
const HEIGHT: u32 = 300;

fn main() -> anyhow::Result<()> {
    let mut prison = prison_architect_savefile::read(r#"cloudsaves\empty.prison"#)?;
    prison.set_property("OriginW", WIDTH.to_string());
    prison.set_property("OriginH", HEIGHT.to_string());
    prison.set_property("NumCellsX", WIDTH.to_string());
    prison.set_property("NumCellsY", HEIGHT.to_string());

    let cells = prison.child_mut("Cells").unwrap();

    let gen = TerrainGen::new(WIDTH as f64, HEIGHT as f64, 2842131717379676695);
    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            let mut cell = Node::new();

            let mat = gen.material(x as f64, y as f64);
            cell.set_property("Mat", mat);
            if mat == "WaterWalkable" {
                cell.set_property("Dep", (gen.depth(x as f64, y as f64) * 0.05).to_string());
            }

            cells.set_child(format!("{} {}", x, y), cell);
        }
    }

    prison.write(r#"saves\after.prison"#)?;
    Ok(())
}

struct Simplex {
    layers: Vec<(f64, f64, noise::OpenSimplex)>,
}

struct TerrainGen {
    dist: Simplex,
    dirt: Simplex,
    sand: Simplex,
    width: f64,
    height: f64,
}

impl Simplex {
    fn new(width: f64, height: f64, seed: u64, count: u32) -> Self {
        let mut rand = SmallRng::seed_from_u64(seed);

        let layers = (0..count as i32)
            .map(|n| {
                let scale = f64::powi(2.0, 4 + n);
                (
                    scale / width,
                    scale / height,
                    noise::OpenSimplex::new(rand.next_u32()),
                )
            })
            .collect();

        Simplex {
            layers,
        }
    }

    fn get(&self, x: f64, y: f64) -> f64 {
        let mut result = 0.0;
        let mut scale = 1.0;
        for (scale_x, scale_y, noise) in &self.layers {
            result += noise.get([x * scale_x, y * scale_y]) * scale;
            scale /= 2.0;
        }
        result
    }
}

impl TerrainGen {
    fn new(width: f64, height: f64, seed: u64) -> Self {
        let dist = Simplex::new(width * 3.0, height * 3.0, seed, 3);
        let dirt = Simplex::new(width, height, 2, 7);
        let sand = Simplex::new(width, height, 3, 7);

        TerrainGen {
            dist,
            dirt,
            sand,
            width,
            height,
        }
    }

    pub fn material(&self, x: f64, y: f64) -> &'static str {
        let dist = f64::hypot(
            scale_to_center(x, self.width),
            scale_to_center(y, self.height),
        );

        let t = (dist + (self.dist.get(x, y)) * 0.55) / 1.05;

        if t < 0.45 {
            if self.dirt.get(x, y) < -0.1 {
                "Dirt"
            } else {
                "LongGrass"
            }
        } else if t < 0.66 {
            if self.dirt.get(x, y) < 0.0 {
                "Dirt"
            } else if self.sand.get(x, y) < 0.0 {
                "Sand"
            } else {
                "LongGrass"
            }
        } else {
            "WaterWalkable"
        }
    }

    pub(crate) fn depth(&self, x: f64, y: f64) -> f64 {
        let dist = f64::hypot(
            scale_to_center(x, self.width),
            scale_to_center(y, self.height),
        );
        let t = (dist + (self.dist.get(x, y)) * 0.55) / 1.05;
        (((t - 0.71) / 0.29) * 1.5).min(1.0).max(0.0)
    }
}

fn scale_to_center(x: f64, w: f64) -> f64 {
    (2.0 * x - w).abs() / w
}
