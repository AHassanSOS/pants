// Copyright 2017 Pants project contributors (see CONTRIBUTORS.md).
// Licensed under the Apache License, Version 2.0 (see LICENSE).

use std::collections::HashMap;
use std::fmt;

use crate::core::{Function, TypeId};
use crate::selectors::{DependencyKey, Get, Select};
use crate::types::Types;

use rule_graph;

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub enum Rule {
  // Intrinsic rules are implemented in rust.
  Intrinsic(Intrinsic),
  // Task rules are implemented in python.
  Task(Task),
}

impl rule_graph::Rule for Rule {
  type TypeId = TypeId;
  type DependencyKey = DependencyKey;

  fn dependency_keys(&self) -> Vec<DependencyKey> {
    match self {
      &Rule::Task(Task {
        ref clause,
        ref gets,
        ..
      }) => clause
        .iter()
        .map(|s| DependencyKey::JustSelect(*s))
        .chain(gets.iter().map(|g| DependencyKey::JustGet(*g)))
        .collect(),
      &Rule::Intrinsic(Intrinsic { ref input, .. }) => {
        vec![DependencyKey::JustSelect(Select::new(*input))]
      }
    }
  }

  fn require_reachable(&self) -> bool {
    match self {
      &Rule::Task(_) => true,
      &Rule::Intrinsic(_) => false,
    }
  }
}

impl fmt::Display for Rule {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match self {
      &Rule::Task(ref task) => {
        let product = format!("{}", task.product);
        let mut clause_portion = task
          .clause
          .iter()
          .map(|c| c.product.to_string())
          .collect::<Vec<_>>()
          .join(", ");
        clause_portion = format!("[{}]", clause_portion);
        let mut get_portion = task
          .gets
          .iter()
          .map(::std::string::ToString::to_string)
          .collect::<Vec<_>>()
          .join(", ");
        get_portion = if task.gets.is_empty() {
          "".to_string()
        } else {
          format!("[{}], ", get_portion)
        };

        write!(
          f,
          "({}, {}, {}{})",
          product, clause_portion, get_portion, task.func,
        )
      }
      &Rule::Intrinsic(ref intrinsic) => write!(
        f,
        "({}, [{}], <intrinsic>)",
        intrinsic.product, intrinsic.input,
      ),
    }
  }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Task {
  pub product: TypeId,
  pub clause: Vec<Select>,
  pub gets: Vec<Get>,
  pub func: Function,
  pub cacheable: bool,
}

///
/// Registry of native (rust) Intrinsic tasks and user (python) Tasks.
///
#[derive(Clone, Debug)]
pub struct Tasks {
  // output product type -> list of rules providing it
  rules: HashMap<TypeId, Vec<Rule>>,
  // Used during the construction of the tasks map.
  preparing: Option<Task>,
}

///
/// A collection of Rules (TODO: rename to Rules).
///
/// Defines a stateful lifecycle for defining tasks via the C api. Call in order:
///   1. task_begin() - once per task
///   2. add_*() - zero or more times per task to add input clauses
///   3. task_end() - once per task
///
/// (This protocol was original defined in a Builder, but that complicated the C lifecycle.)
///
impl Tasks {
  pub fn new() -> Tasks {
    Tasks {
      rules: HashMap::default(),
      preparing: None,
    }
  }

  pub fn as_map(&self) -> &HashMap<TypeId, Vec<Rule>> {
    &self.rules
  }

  pub fn intrinsics_set(&mut self, types: &Types) {
    let intrinsics = vec![
      Intrinsic {
        product: types.directory_digest,
        input: types.input_files_content,
      },
      Intrinsic {
        product: types.snapshot,
        input: types.path_globs,
      },
      Intrinsic {
        product: types.snapshot,
        input: types.url_to_fetch,
      },
      Intrinsic {
        product: types.snapshot,
        input: types.directory_digest,
      },
      Intrinsic {
        product: types.files_content,
        input: types.directory_digest,
      },
      Intrinsic {
        product: types.directory_digest,
        input: types.directories_to_merge,
      },
      Intrinsic {
        product: types.directory_digest,
        input: types.directory_with_prefix_to_strip,
      },
      Intrinsic {
        product: types.process_result,
        input: types.multi_platform_process_request,
      },
    ];

    for intrinsic in intrinsics {
      self.insert_rule(intrinsic.product, Rule::Intrinsic(intrinsic))
    }
  }

  ///
  /// The following methods define the Task registration lifecycle.
  ///
  pub fn task_begin(&mut self, func: Function, product: TypeId, cacheable: bool) {
    assert!(
      self.preparing.is_none(),
      "Must `end()` the previous task creation before beginning a new one!"
    );

    self.preparing = Some(Task {
      cacheable: cacheable,
      product: product,
      clause: Vec::new(),
      gets: Vec::new(),
      func: func,
    });
  }

  pub fn add_get(&mut self, product: TypeId, subject: TypeId) {
    self
      .preparing
      .as_mut()
      .expect("Must `begin()` a task creation before adding gets!")
      .gets
      .push(Get {
        product: product,
        subject: subject,
      });
  }

  pub fn add_select(&mut self, product: TypeId) {
    self
      .preparing
      .as_mut()
      .expect("Must `begin()` a task creation before adding clauses!")
      .clause
      .push(Select::new(product));
  }

  pub fn task_end(&mut self) {
    // Move the task from `preparing` to the Rules map
    let task = self
      .preparing
      .take()
      .expect("Must `begin()` a task creation before ending it!");
    self.insert_rule(task.product, Rule::Task(task))
  }

  fn insert_rule(&mut self, product: TypeId, rule: Rule) {
    let rules = self.rules.entry(product).or_insert_with(Vec::new);
    assert!(
      !rules.contains(&rule),
      "{:?} was double-registered for {:?}: {:?}",
      rule,
      product,
      rules,
    );
    rules.push(rule);
  }
}

#[derive(Eq, Hash, PartialEq, Clone, Copy, Debug)]
pub struct Intrinsic {
  pub product: TypeId,
  pub input: TypeId,
}
