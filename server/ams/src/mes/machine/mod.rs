use std::cell::RefCell;

use recipe::Recipe;

mod recipe;

#[derive(Debug)]
enum Error {
    State,
}

#[derive(Debug)]
enum Status {
    StandBy,
    Processing { product_id: i32 },
    Offline,
}

struct Machine {
    id: String,
    status: RefCell<Status>,
    recipe: RefCell<Vec<Recipe>>,
}

impl Machine {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            status: RefCell::new(Status::Offline),
            recipe: RefCell::new(Vec::new()),
        }
    }

    pub fn inline(&self, recipes: Vec<Recipe>) -> Result<(), Error> {
        let mut state = self.status.borrow_mut();
        match *state {
            Status::Offline | Status::StandBy => {
                *state = Status::StandBy;
                *self.recipe.borrow_mut() = recipes
            }
            Status::Processing { product_id: _ } => return Err(Error::State),
        }
        Ok(())
    }

    pub fn process(&self, product_id: i32) -> Result<(), Error> {
        let mut state = self.status.borrow_mut();
        match *state {
            Status::StandBy => *state = Status::Processing { product_id },
            Status::Offline | Status::Processing { product_id: _ } => {
                return Err(Error::State);
            }
        }
        Ok(())
    }

    pub fn process_done(&self) -> Result<(), Error> {
        let mut state = self.status.borrow_mut();
        match *state {
            Status::Processing { product_id: _ } => *state = Status::StandBy,
            Status::Offline | Status::StandBy => {
                return Err(Error::State);
            }
        }
        Ok(())
    }
}
