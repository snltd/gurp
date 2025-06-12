use crate::common::types::Opts;
use std::fmt::Display;

// TODO settle on what we're going to do with the opts. We could write machine-parseable output, or
// colour, or write selectively based on --debug, --verbose, or even a log-level.

pub struct Output {
    doer: String,
    opts: Opts,
}

impl Output {
    pub fn new(doer: &str, opts: &Opts) -> Self {
        Self {
            doer: doer.to_owned(),
            opts: opts.clone(),
        }
    }

    pub fn creating<T: Display>(&self, item_name: T) {
        println!("[{}::{}] CREATING", self.doer, item_name);
    }

    pub fn removing<T: Display>(&self, item_name: T) {
        println!("[{}::{}] REMOVING", self.doer, item_name);
    }

    pub fn action<T: Display>(&self, item_name: T, action: &str) {
        println!("[{}::{}] {}", self.doer, item_name, action);
    }

    pub fn change<T: Display, U: Display>(&self, item_name: T, from: &U, to: &U) {
        println!(
            "[{}::{}] CHANGE '{}' -> '{}'",
            self.doer, item_name, from, to
        );
    }

    pub fn change_name_only<T: Display>(&self, item_name: T) {
        println!("[{}::{}] CHANGE", self.doer, item_name);
    }

    pub fn no_change<T: Display>(&self, item_name: T) {
        println!("[{}::{}] NO CHANGE", self.doer, item_name);
    }

    pub fn not_present<T: Display>(&self, item_name: T) {
        println!("[{}::{}] NOT PRESENT", self.doer, item_name);
    }

    pub fn protected<T: Display>(&self, item_name: T) {
        println!("[{}::{}] PROTECTED", self.doer, item_name);
    }
}
