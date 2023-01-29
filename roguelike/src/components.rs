use specs::prelude::*;
use specs_derive::*;

#[derive(Component, Debug)]
pub struct Name {
    pub name: String,
}

#[derive(Component, Debug)]
pub struct Monster;
