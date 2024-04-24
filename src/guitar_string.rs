use rand::distributions::Uniform;
use rand::prelude::*;
use std::collections::VecDeque;

const DECAY_FACTOR: f64 = 0.996;

pub struct GuitarString {
    m_ring_buffer: VecDeque<f64>,
    m_capacity: usize,
    m_time: usize,

    //uniform distribution and random number generator
    m_rng: ThreadRng,
    m_uniform: Uniform<f64>,
}

impl GuitarString {
    pub fn new(frequency: f64) -> GuitarString {
        let capacity = (44100.0 / frequency) as usize;
        let mut ring_buffer = VecDeque::with_capacity(capacity);
        for _ in 0..capacity {
            ring_buffer.push_back(0.0);
        }

        // create a random number generator
        let rng = thread_rng();
        // create a uniform distribution between -0.5 and 0.5
        let uniform = Uniform::from(-0.5..=0.5);

        GuitarString {
            m_ring_buffer: ring_buffer,
            m_capacity: capacity,
            m_time: 0,
            m_rng: rng,
            m_uniform: uniform,
        }
    }

    pub fn pluck(&mut self) {
        // fill the ring buffer with random values
        for i in 0..self.m_capacity {
            self.m_ring_buffer[i] = self.m_rng.sample(self.m_uniform);
        }
    }

    pub fn tic(&mut self) {
        let first = self.m_ring_buffer.pop_front().unwrap();
        let second = self.m_ring_buffer.front().unwrap();
        let average = (first + second) / 2.0;

        self.m_ring_buffer.push_back(average * DECAY_FACTOR);

        self.m_time += 1;
    }

    pub fn sample(&self) -> f64 {
        self.m_ring_buffer.front().unwrap().clone()
    }
}

