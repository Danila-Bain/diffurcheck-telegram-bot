

#[derive(Debug, Clone, Default)]
pub enum AssignmentState{
    #[default]
    AwaitingSolutions,
    ConfirmCompletion,
}
