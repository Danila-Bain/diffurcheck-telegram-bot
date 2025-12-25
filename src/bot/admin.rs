use teloxide::dptree::{self, Handler};

use crate::bot::HandlerResult;

#[derive(Debug, Clone)]
pub enum AdminState {
    AwaitingCommand,
}

// pub fn admin_handler()
// -> Handler<'static, HandlerResult, teloxide::dispatching::DpHandlerDescription> {
//     dptree::entry()
//     // dptree::case![BotState::Idle(idle_state)]
//     //     .branch(
//     //         Update::filter_message().branch(
//     //             dptree::case![IdleState::AwaitingCommand { student_id }]
//     //                 .branch(
//     //                     teloxide::filter_command::<IdleCommand, HandlerResult>()
//     //                         .branch(dptree::case![IdleCommand::Help].endpoint(help))
//     //                         .branch(
//     //                             dptree::case![IdleCommand::ShowAssignments]
//     //                                 .endpoint(show_assignments_list),
//     //                         ),
//     //                 )
//     //                 .branch(dptree::endpoint(unknown_command)),
//     //         ),
//     //     )
//     //     .branch(
//     //         Update::filter_callback_query()
//     //             .branch(
//     //                 dptree::case![IdleState::AwaitingCommand { student_id }]
//     //                     .endpoint(show_assignment),
//     //             )
//     //             .branch(
//     //                 dptree::case![IdleState::AwaitingAssignmentStart {
//     //                     student_id,
//     //                     group_assignment_id
//     //                 }]
//     //                 .endpoint(awaiting_assignment_start),
//     //             ),
//     //     )
// }
