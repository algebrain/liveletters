/// Аргументы команды `lltt cu`.
///
/// Семантика разбора первого аргумента определена в [`crate::run`]:
/// - без аргументов — показать текущую идентичность;
/// - первый аргумент совпадает с подкомандой (`list`, `show`, `add`, `rm`) — выполнить её;
/// - первый аргумент — что-то иное — переключить текущую идентичность.
#[derive(Debug, clap::Args)]
pub struct Args {
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, num_args = 0..)]
    pub tokens: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CuAction {
    Current,
    Switch {
        name: String,
    },
    ShowCurrent {
        reveal: bool,
    },
    Posts {
        limit: Option<usize>,
    },
    List,
    Init {
        name: String,
        force: bool,
    },
    Show {
        name: String,
        reveal: bool,
    },
    Add {
        name: String,
        from: Option<std::path::PathBuf>,
    },
    Rm {
        name: String,
        yes: bool,
    },
}
