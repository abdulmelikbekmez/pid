use crate::write_once::WriteOnce;

pub struct PID {
    err: f32,
    err_prev: f32,

    err_integral: f32,
    err_derivative: f32,
    saturated: bool,

    max_output: f32,

    pub k_p: WriteOnce<f32>,
    pub k_i: WriteOnce<f32>,
    pub k_d: WriteOnce<f32>,
}

impl PID {
    pub fn new(k_p: WriteOnce<f32>, k_i: WriteOnce<f32>, k_d: WriteOnce<f32>) -> Self {
        Self {
            k_p,
            k_d,
            k_i,
            err: 0.0,
            err_prev: 0.0,
            err_integral: 0.0,
            err_derivative: 0.0,
            saturated: false,
            max_output: 100.0,
        }
    }
    pub fn main(&mut self, error: f32, dt: f32) -> f32 {
        self.update(error, dt);
        let output = self.output_p() + self.output_i() + self.output_d();
        self.saturated = output.abs() >= self.max_output;

        output.clamp(-self.max_output, self.max_output)
    }

    fn output_p(&self) -> f32 {
        self.err * *self.k_p
    }

    fn output_i(&self) -> f32 {
        self.err_integral * *self.k_i
    }

    fn output_d(&self) -> f32 {
        self.err_derivative * *self.k_d
    }

    fn update(&mut self, error: f32, dt: f32) {
        self.err_prev = self.err;
        self.err = error;

        self.err_derivative = (self.err - self.err_prev) / dt;

        if !self.saturated {
            self.err_integral += self.err * dt;
        }
    }
}
