use uuid::Uuid;
use std::path::PathBuf;

pub async fn compile_assignment_pdf(
    _assignment_id: Uuid,
) -> PathBuf {
    // 1. generate Typst source
    // 2. include student tables
    // 3. include problems and solutions
    // 4. run typst compile

    PathBuf::from("compiled/assignment.pdf");

    todo!();
}
