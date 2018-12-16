use std::default::Default;
use std::fmt::Display;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time;

pub struct BusReader<T: Display + Default + Clone> {
    buffer: Arc<Vec<AtomicPtr<T>>>,
    wi: Arc<AtomicUsize>,
    ri: AtomicUsize,
    size: usize,
}
impl<T: Display + Default + Clone> BusReader<T> {
    pub fn recv(&self) -> Option<T> {
        if self.ri.load(Ordering::Relaxed) == self.wi.load(Ordering::Relaxed) {
            return None;
        }
        let mut object;
        loop {
            let temp = self
                .buffer
                .get(self.ri.load(Ordering::Relaxed) % self.size)
                .unwrap();
            object = unsafe { &*temp.load(Ordering::Relaxed) };
            if self.wi.load(Ordering::Relaxed) > self.ri.load(Ordering::Relaxed) + self.size {
                self.ri.store(
                    self.wi.load(Ordering::Relaxed) - self.size,
                    Ordering::Relaxed,
                );
            } else {
                self.ri.fetch_add(1, Ordering::Relaxed);
                return Some(object.clone());
            }
        }
    }
}
pub struct Bus<T: Display + Default + Clone> {
    // atp to an array of atps of option<arc<t>>
    buffer: Arc<Vec<AtomicPtr<T>>>,
    wi: Arc<AtomicUsize>,
    size: usize,
}

impl<T: Display + Default + Clone> Bus<T> {
    pub fn new(size: usize) -> Self {
        let mut temp: Vec<AtomicPtr<T>> = Vec::new();
        for _i in 0..size {
            temp.push(AtomicPtr::new(&mut T::default()));
        }

        println!("*****new********{}", temp.len());
        for (index, object) in (&temp).into_iter().enumerate() {
            let x = unsafe { &*object.load(Ordering::Relaxed) };
            println!("{} : Some({})", index, x);
        }
        println!("*****new********{}", temp.len());

        Self {
            buffer: Arc::new(temp),
            wi: Arc::new(AtomicUsize::new(0)),
            size: size,
        }
    }
    pub fn add_sub(&self) -> BusReader<T> {
        BusReader {
            buffer: self.buffer.clone(),
            wi: self.wi.clone(),
            ri: AtomicUsize::new(0),
            size: self.size,
        }
    }
    pub fn push(&self, object: &mut T) {
        //println!("pushed");

        let temp = &*self.buffer;

        let temp = temp
            .get(self.wi.load(Ordering::Relaxed) % self.size)
            .unwrap();
        temp.store(object, Ordering::Relaxed);
        self.wi.fetch_add(1, Ordering::Relaxed);
    }
    pub fn print(&self) {
        let temp = &*self.buffer;
        println!("******print********{}", temp.len());
        for (index, object) in temp.into_iter().enumerate() {
            let me = unsafe { &*object.load(Ordering::Relaxed) }.clone();
            println!("{} : Some({})", index, me);
        }
        println!("******print********");
    }
}

fn main() {
    let bus: Bus<u32> = Bus::new(10);
    let rx1 = bus.add_sub();
    let rx2 = bus.add_sub();
    let a = thread::spawn(move || {
        let mut vec = Vec::new();
        for i in 0..40 {
            vec.push(i);
        }
        thread::sleep(time::Duration::from_millis(2000));
        for i in &mut vec {
            bus.push(i);
            //bus.print();
            thread::sleep(time::Duration::from_millis(500));
        }
    });

    let b = thread::spawn(move || {
        thread::sleep(time::Duration::from_millis(1000));
        for _i in 0..100 {
            match rx1.recv() {
                None => println!("b: Got none weird!"),
                Some(ref arc_obj) => println!("b: {}", arc_obj),
            }
            thread::sleep(time::Duration::from_millis(100));
        }
    });

    let c = thread::spawn(move || {
        thread::sleep(time::Duration::from_millis(1000));
        for _i in 0..100 {
            match rx2.recv() {
                None => println!("c: Got none weird!"),
                Some(ref arc_obj) => println!("c: {}", arc_obj),
            }
            thread::sleep(time::Duration::from_millis(1000));
        }
    });
    a.join();
    b.join();
    c.join();
}