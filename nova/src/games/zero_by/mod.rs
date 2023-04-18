//! # Zero-By Game Module
//!
//! Zero-By is a small acyclic game, where two players take turns removing 
//! one of certain amounts of elements from a set of N elements. For example,
//! players could take turns removing either one or two coins from a stack
//! of ten, which would be an instance of Ten to Zero by One or Two (coins).
//! 
//! This module encapsulates the commonalities for all Zero-By games, allowing
//! users to specify which abstract instance of the Zero-By game they wish to
//! emulate.
//!
//! #### Authorship
//!
//! - Max Fierro, 4/6/2023 (maxfierro@berkeley.edu)

/* INFRA IMPORTS */

use super::{AcyclicGame, Game, SmallGame, GameInformation};
use crate::core::{
    solvers::{acyclic::AcyclicSolver, cyclic::CyclicSolver, tiered::TierSolver},
    solvers::{AcyclicallySolvable, CyclicallySolvable, Solvable, TierSolvable},
    State, Value, Variant,
};
use crate::implement;
use std::collections::HashSet;
use std::process;
use regex::Regex;

implement! { for Session =>
    AcyclicGame,
    TierSolvable,
    AcyclicallySolvable,
    CyclicallySolvable,
    SmallGame
}

/* CONSTANTS */

const NAME: &str = "Zero-By";
const AUTHOR: &str = "Max Fierro";
const DESCRIPTION: &str = 
"Two players take turns removing a number of elements from a set of arbitrary
size. They can make a choice of how many elements to remove (and of how many
elements to start out with) based on the game variant. The player who is left
with 0 elements in their turn loses. A player cannot remove more elements than
currently available in the set.";

const VARIANT_DEFAULT: &str = "10-1-2";
const VARIANT_PATTERN: &str = r"^\d+(?:-\d+)*$";

/* GAME IMPLEMENTATION */

/// Represents a Zero-By game instance.
pub struct Session {
    variant: Option<String>,
    from: State,
    by: Vec<u64>
}

fn decode_variant(v: Variant) -> Session {
    let re = Regex::new(VARIANT_PATTERN).unwrap();
    if !re.is_match(&v) {
        println!("Variant string malformed.");
        process::exit(exitcode::USAGE);
    }
    let mut from_by = v.split("-")
        .map(|int_string| {
            int_string.parse::<u64>()
                .expect("Could not parse variant.")
        })
        .collect::<Vec<u64>>();
    Session {
        variant: Some(v),
        from: {
            if let Some(from) = from_by.get(0) {
                from.clone().into()
            } else {
                panic!("Could not parse variant.")
            }
        },
        by: {
            from_by.remove(0);
            from_by
        }
    }
}

impl Game for Session {
    fn initialize(variant: Option<Variant>) -> Self {
        if let Some(variant) = variant {
            decode_variant(variant)
        } else {
            decode_variant(VARIANT_DEFAULT.to_owned())
        }
    }

    fn start(&self) -> State {
        self.from
    }

    fn adjacent(&self, state: State) -> HashSet<State> {
        let mut children = HashSet::new();
        for choice in self.by.iter() {
            if state >= *choice {
                children.insert(state - choice);
            }
        }
        children
    }

    fn value(&self, state: State) -> Option<Value> {
        if state > 0 {
            None
        } else {
            Some(Value::Lose(0))
        }
    }

    fn id(&self) -> String {
        if let Some(variant) = self.variant.clone() {
            format!("{}.{}", NAME, variant)
        } else {
            NAME.to_owned()
        }
    }

    fn info(&self) -> GameInformation {
        GameInformation {
            name:            NAME.to_owned(),
            author:          AUTHOR.to_owned(),
            description:     DESCRIPTION.to_owned(),
            variant_pattern: VARIANT_PATTERN.to_owned(),
            variant_default: VARIANT_DEFAULT.to_owned(),
        }
    }
}

impl Solvable for Session {
    fn solvers(&self) -> Vec<(Option<String>, fn(&Self, bool, bool) -> Value)> {
        vec![
            (None,                              Self::acyclic_solve),
            (Some(Self::acyclic_solver_name()), Self::acyclic_solve),
            (Some(Self::cyclic_solver_name()),  Self::cyclic_solve),
            (Some(Self::tier_solver_name()),    Self::tier_solve),
        ]
    }
}
