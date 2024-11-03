pub fn is_unique_constraint_violation(err: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(db_err) = err {
        db_err.is_unique_violation()
    } else {
        false
    }
}
