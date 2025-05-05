use std::rc::Rc;

use crate::app::App;

#[derive(Clone)]
pub struct Command<T> {
    #[allow(clippy::type_complexity)]
    action: Rc<dyn Fn(&mut App, T)>,
}

impl<T> Command<T> {
    pub fn new(action: impl Fn(&mut App, T) + 'static) -> Self {
        Self {
            action: Rc::new(action),
        }
    }

    pub fn run(&self, app: &mut App, arg: T) {
        (self.action)(app, arg)
    }
}
