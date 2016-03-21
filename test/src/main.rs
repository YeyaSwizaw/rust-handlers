#![feature(plugin, box_syntax)]
#![plugin(handlers)]

define_handler_system! System {
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

impl SystemObject for Test {
    fn as_MouseHandler(&self) -> Option<&MouseHandler> { None }
    fn as_MouseHandler_mut(&mut self) -> Option<&mut MouseHandler> { None }

    fn as_InputHandler(&self) -> Option<&InputHandler> { Some(self as &InputHandler) }
    fn as_InputHandler_mut(&mut self) -> Option<&mut InputHandler> { Some(self as &mut InputHandler) }
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
