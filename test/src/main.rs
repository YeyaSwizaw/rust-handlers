#![feature(plugin, specialization, box_syntax)]
#![plugin(interpolate_idents)]

#[macro_use] extern crate handlers;

pub trait Renderable {
    fn render(&self);
    fn update(&mut self, x: i64);
}

handlers! {
    System {
        MouseHandler {
            click(x: u64, y: u64) => on_click;
            hover() => on_hover
        }

        InputHandler {
            input(input: char) => on_input
        }
    }
}

pub struct Test {
    pub n: i64
}

impl InputHandler for Test {
    fn on_input(&mut self, input: char) {
        println!("{}: {}", self.n, input);
        self.n = self.n + 1;
    }
}

impl Renderable for Test {
    fn render(&self) {
        println!("Rendering! {}", self.n);
    }

    fn update(&mut self, x: i64) {
        self.n += x;
    }
}

handlers_objects! {
    System {
        Test
    }
}

fn main() {
    let mut system = System::new();
    let idx = system.add(box Test{n: 15});
    // for obj in system.iter() { obj.render(); }
    system.input('H');
    system.input('e');
    system.add(box Test{n: 20});
    // for obj in system.iter() { obj.render(); }
    system.input('l');
    system.hover();
    system.input('l');
    let obj = system.remove(idx).unwrap();
    //obj.render();
    // for obj in system.iter() { obj.render(); }
    system.input('o');
    system.input('!');
    system.add(box Test{n: 25});
    // for obj in system.iter() { obj.render(); }
    // for obj in system.iter_mut() { obj.update(-10); obj.render(); }
}
