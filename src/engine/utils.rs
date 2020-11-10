pub fn print_warning<T: core::fmt::Debug, S: core::fmt::Debug>(t: &T, s: &S) {
    eprintln!("WARNING: Ignored transaction {:#?} for client {:#?}", t, s);
}
