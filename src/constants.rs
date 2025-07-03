use std::cell::LazyCell;
use std::collections::HashSet;
use std::rc::Rc;

pub const NUMERICS: LazyCell<Rc<HashSet<char>>> =
    LazyCell::new(|| HashSet::from_iter('0'..='9').into());

pub const LOWERCASE_ENGLISH: LazyCell<Rc<HashSet<char>>> =
    LazyCell::new(|| HashSet::from_iter('a'..='z').into());

pub const UPPERCASE_ENGLISH: LazyCell<Rc<HashSet<char>>> =
    LazyCell::new(|| HashSet::from_iter('A'..='Z').into());
