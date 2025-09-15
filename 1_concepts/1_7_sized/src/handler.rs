use crate::user::{Storage, User};

trait CommandHandler<C> {
    type Context: ?Sized;
    type Result;

    fn handle_command(&self, cmd: &C, ctx: &Self::Context) -> Self::Result;
}
