use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::{expr::LiteralValue, panic::PanicHandler};

#[derive(Clone)]
pub struct Environment {
    pub values: Rc<RefCell<HashMap<String, LiteralValue>>>,
    pub locals: Rc<RefCell<HashMap<usize, usize>>>,
    pub enclosing: Option<Rc<Environment>>,
}

impl Environment {
    pub fn new(locals: HashMap<usize, usize>) -> Self {
        Self {
            values: Rc::new(RefCell::new(HashMap::new())),
            locals: Rc::new(RefCell::new(locals)),
            enclosing: None,
        }
    }

    pub fn resolve(&self, locals: HashMap<usize, usize>) {
        locals.iter().for_each(|(key, val)| {
            self.locals.borrow_mut().insert(*key, *val);
        });
    }

    pub fn get_value(&self, name: String) -> Option<LiteralValue> {
        self.values.borrow().get(&name).cloned()
    }

    pub fn enclose(&self) -> Environment {
        Self {
            values: Rc::new(RefCell::new(HashMap::new())),
            locals: self.locals.clone(),
            enclosing: Some(Rc::new(self.clone())),
        }
    }

    pub fn define(&self, name: &str, value: LiteralValue) {
        self.values.borrow_mut().insert(name.to_string(), value);
    }

    pub fn constant(&self, name: &str) -> bool {
        self.values
            .borrow()
            .contains_key(format!("__const__{}", name).as_str())
    }

    pub fn get(&self, name: &str, id: usize) -> Option<LiteralValue> {
        self.internal(name, self.locals.borrow().get(&id).cloned())
    }

    pub fn get_this_instance(&self, id: usize) -> Option<LiteralValue> {
        let distance: usize = self.locals.borrow().get(&id).cloned().unwrap_or_else(|| {
            PanicHandler::new(
                None,
                None,
                None,
                "Could not find 'this' even though 'super' was defined.",
            )
            .panic();

            0
        });

        self.internal("this", Some(distance - 1))
    }

    fn internal(&self, name: &str, distance: Option<usize>) -> Option<LiteralValue> {
        if distance.is_none() {
            match &self.enclosing {
                None => {
                    let const_i: String = format!("__const__{}", name);

                    if !self.values.borrow().contains_key(const_i.as_str()) {
                        return self.values.borrow().get(name).cloned();
                    }

                    self.values.borrow().get(const_i.as_str()).cloned()
                }
                Some(env) => env.internal(name, distance),
            }
        } else {
            let distance: usize = distance.unwrap();
            if distance == 0 {
                self.values.borrow().get(name).cloned()
            } else {
                match &self.enclosing {
                    None => {
                        PanicHandler::new(
                            None,
                            None,
                            None,
                            format!(
                                "Could not find variable ({}) at distance ({}).",
                                name, distance
                            )
                            .as_str(),
                        )
                        .panic();
                        unreachable!()
                    }
                    Some(env) => env.internal(name, Some(distance - 1)),
                }
            }
        }
    }

    pub fn assign_global(&self, name: &str, value: &LiteralValue) -> bool {
        self.assign_internal(name, value, None)
    }

    pub fn assign(&self, name: &str, value: &LiteralValue, id: usize) -> bool {
        self.assign_internal(name, value, self.locals.borrow().get(&id).cloned())
    }

    fn assign_internal(&self, name: &str, value: &LiteralValue, distance: Option<usize>) -> bool {
        if distance.is_none() {
            match &self.enclosing {
                Some(env) => env.assign_internal(name, value, distance),
                None => self
                    .values
                    .borrow_mut()
                    .insert(name.to_string(), value.to_owned())
                    .is_some(),
            }
        } else {
            if distance.unwrap() == 0 {
                self.values
                    .borrow_mut()
                    .insert(name.to_string(), value.to_owned());
                return true;
            }

            match &self.enclosing {
                None => {
                    PanicHandler::new(
                        None,
                        None,
                        None,
                        format!(
                            "Could not find variable ({}) at distance ({}).",
                            name,
                            distance.unwrap()
                        )
                        .as_str(),
                    )
                    .panic();

                    false
                }
                Some(env) => env.assign_internal(name, value, Some(distance.unwrap() - 1)),
            };
            true
        }
    }
}
