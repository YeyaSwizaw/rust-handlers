#![feature(plugin)]
#![plugin(handlers)]

pub trait Renderable {}

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

impl Renderable for Test {}

handlers_impl_object! System { 
    Test: InputHandler 
}

fn main() {
    let mut system = System::new();
    system.add(Test { n: 15 });
    system.input('H');
    system.input('e');
    system.input('l');
    system.hover();
    system.input('l');
    system.add(Test { n: 20 });
    system.input('o');
    system.input('!');
}
