use uuid::Uuid;
use teloxide::types::InputFile;

pub struct GeneratedProblem {
    pub id: Uuid,
    pub image: InputFile,
}

pub async fn generate_problems(
    _assignment_id: Uuid,
    _student_id: Uuid,
) -> Vec<GeneratedProblem> {
    // TODO: actual generation
    vec![]
}
