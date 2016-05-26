use super::GameAction;

pub trait CommandBuilder<Cmd> {
    fn get_command(&mut self) -> Cmd;
}
