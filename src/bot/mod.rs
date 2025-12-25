use teloxide::{
    dispatching::dialogue::{self, InMemStorage},
    dptree::Handler,
    prelude::Dialogue,
    types::Update,
};

pub type MyDialogue = Dialogue<BotState, InMemStorage<BotState>>;
pub type MyError = Box<dyn std::error::Error + Send + Sync>;
pub type HandlerResult = Result<(), MyError>;

pub mod assignment;
pub mod idle;
pub mod registration;
pub mod start;
pub mod admin;

#[derive(Debug, Clone, Default)]
pub enum BotState {
    #[default]
    Start,
    Registration(registration::RegistrationState),
    Idle(idle::IdleState),
    Assignment(assignment::AssignmentState),
    Admin(admin::AdminState),
}

pub fn main_handler() -> Handler<'static, HandlerResult, teloxide::dispatching::DpHandlerDescription>
{
    dialogue::enter::<Update, InMemStorage<BotState>, BotState, _>()
        .branch(start::start_handler())
        .branch(registration::registration_handler())
        .branch(idle::idle_handler())
        .branch(assignment::assignment_handler())
}
