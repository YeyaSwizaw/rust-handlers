#![feature(plugin)]
#![plugin(handlers)]

pub trait Renderable {
    fn render(&self);
    fn update(&mut self, x: i64);
}

handlers_define_system! System {
    * : Renderable;

    MouseHandler {
        click(x: u64, y: u64) => on_click;
        hover() => on_hover
    }

    InputHandler {
        input(input: char) => on_input
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

handlers_impl_object! System { 
    Test: InputHandler 
}

fn main() {
    let mut system = System::new();
    system.add(Test { n: 15 });
    for obj in system.iter() {
        obj.render();
    }
    system.input('H');
    system.input('e');
    system.input('l');
    system.hover();
    system.input('l');
    system.add(Test { n: 20 });
    system.input('o');
    system.input('!');
    for obj in system.iter_mut() {
        obj.update(-10);
        obj.render();
    }
}
