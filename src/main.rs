#![feature(const_panic)]
#![feature(min_const_generics)]

#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

#[macro_use]
extern crate lazy_static;

mod db;
mod engine;
mod output;
mod parser;
fn main() {
    let path: String = std::env::args()
        .collect::<Vec<String>>()
        .get(1)
        .expect("no path")
        .parse()
        .expect("invalid path");

    let mut db = db::Db::new();

    parser::pre_parse(path.clone(), &mut db);

    parser::parse(path, &mut db);

    output::OutputRow::print_header();

    for (key, value) in db.get_clients() {
        let output_row: output::OutputRow = value.into();
        output_row.print(key);
    }
}
