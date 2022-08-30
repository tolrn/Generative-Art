use super::blur::Blur;
use super::population_config::PopulationConfig;
use rand::Rng;

use rand::distributions::Uniform;

/// A 2D grid with a scalar value per each grid block. Each grid is occupied by a single population,
/// hence we store the population config inside the grid.
#[derive(Debug)]
pub struct Grid {
    pub config: PopulationConfig,
    pub width: usize,
    pub height: usize,

    data: Vec<f32>,

    // Scratch space for the blur operation.
    buf: Vec<f32>,
    blur: Blur,
}

impl Grid {
    /// Create a new grid filled with random floats in the [0.0..1.0) range.
    pub fn new<R: Rng + ?Sized>(width: usize, height: usize, rng: &mut R) -> Self {
        if !width.is_power_of_two() || !height.is_power_of_two() {
            panic!("Grid dimensions must be a power of two.");
        }
        let range = Uniform::from(0.0..1.0);
        let data = rng.sample_iter(range).take(width * height).collect();

        Grid {
            width,
            height,
            data,
            config: PopulationConfig::new(rng),
            buf: vec![0.0; width * height],
            blur: Blur::new(width),
        }
    }

    /// Truncate x and y and return a corresponding index into the data slice.
    fn index(&self, x: f32, y: f32) -> usize {
        // x/y can come in negative, hence we shift them by width/height.
        let i = (x + self.width as f32) as usize & (self.width - 1);
        let j = (y + self.height as f32) as usize & (self.height - 1);
        j * self.width + i
    }

    /// Get the buffer value at a given position. The implementation effectively treats data as
    /// periodic, hence any finite position will produce a value.
    pub fn get_buf(&self, x: f32, y: f32) -> f32 {
        self.buf[self.index(x, y)]
    }

    /// Add a value to the grid data at a given position.
    pub fn deposit(&mut self, x: f32, y: f32) {
        let idx = self.index(x, y);
        self.data[idx] += self.config.deposition_amount;
    }

    /// Diffuse grid data and apply a decay multiplier.
    pub fn diffuse(&mut self, radius: usize) {
        self.blur.run(
            &mut self.data,
            &mut self.buf,
            self.width,
            self.height,
            radius as f32,
            self.config.decay_factor,
        );
    }

    pub fn quantile(&self, fraction: f32) -> f32 {
        let index = if (fraction - 1.0_f32).abs() < f32::EPSILON {
            self.data.len() - 1
        } else {
            (self.data.len() as f32 * fraction) as usize
        };
        let mut sorted = self.data.clone();
        sorted
            .as_mut_slice()
            .select_nth_unstable_by(index, |a, b| a.partial_cmp(b).unwrap());
        sorted[index]
    }

    pub fn data(&self) -> &[f32] {
        &self.data
    }
}

pub fn combine<T>(grids: &mut [Grid], attraction_table: &[T])
where
    T: AsRef<[f32]> + Sync,
{
    let datas: Vec<_> = grids.iter().map(|grid| &grid.data).collect();
    let bufs: Vec<_> = grids.iter().map(|grid| &grid.buf).collect();

    // We mutate grid buffers and read grid data. We use unsafe because we need shared/unique
    // borrows on different fields of the same Grid struct.
    bufs.iter().enumerate().for_each(|(i, buf)| unsafe {
        let buf_ptr = *buf as *const Vec<f32> as *mut Vec<f32>;
        buf_ptr.as_mut().unwrap().fill(0.0);
        datas.iter().enumerate().for_each(|(j, other)| {
            let multiplier = attraction_table[i].as_ref()[j];
            buf_ptr
                .as_mut()
                .unwrap()
                .iter_mut()
                .zip(*other)
                .for_each(|(to, from)| *to += from * multiplier)
        })
    });
}