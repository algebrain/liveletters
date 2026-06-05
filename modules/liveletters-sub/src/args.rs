/// Аргументы команды `lltt sub`.
///
/// Семантика разбора первого аргумента:
/// - без аргументов — ошибка (`InvalidArgs`);
/// - первый аргумент совпадает с подкомандой (`list`, `rm`) — выполнить её;
/// - первый аргумент — почтовый адрес — подписаться на блог по этому адресу.
#[derive(Debug, clap::Args)]
pub struct Args {
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, num_args = 0..)]
    pub tokens: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubAction {
    Subscribe { resource_address: String },
    List,
    Rm { resource_address: String },
}
