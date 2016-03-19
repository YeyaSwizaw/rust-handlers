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

/*
pub trait MouseHandler {
    fn on_click(&mut self);
}

pub trait InputHandler {
    fn on_input(&mut self, input: char);
}

pub trait SystemObject {
    fn as_MouseHandler(&self) -> Option<&MouseHandler>;
    fn as_MouseHandler_mut(&mut self) -> Option<&mut MouseHandler>;

    fn as_InputHandler(&self) -> Option<&InputHandler>;
    fn as_InputHandler_mut(&mut self) -> Option<&mut InputHandler>;
}

pub struct System {
    objects: Vec<Box<SystemObject>>,

    MouseHandler_idxs: Vec<usize>,
    InputHandler_idxs: Vec<usize>,
}

impl System {
    pub fn new() -> System {
        System {
            objects: Vec::new(),

            MouseHandler_idxs: Vec::new(),
            InputHandler_idxs: Vec::new(),
        }
    }

    pub fn add<O>(&mut self, object: O) where O: 'static + SystemObject {
        let idx = self.objects.len();

        self.objects.push(Box::new(object));
        let object = self.objects.last().unwrap();

        if let Some(_) = object.as_MouseHandler() {
            self.MouseHandler_idxs.push(idx);
        }

        if let Some(_) = object.as_InputHandler() {
            self.InputHandler_idxs.push(idx);
        }
    }
}
*/

pub struct Test {
    pub n: i64
}

impl SystemObject for Test {
    fn as_MouseHandler(&self) -> Option<&MouseHandler> { None }
    fn as_MouseHandler_mut(&mut self) -> Option<&mut MouseHandler> { None }

    fn as_InputHandler(&self) -> Option<&InputHandler> { None }
    fn as_InputHandler_mut(&mut self) -> Option<&mut InputHandler> { None }
}

fn main() {
    let mut system = System::new();
    system.add(Test { n: 15 });
}
