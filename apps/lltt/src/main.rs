use std::process::ExitCode;

use clap::{Parser, Subcommand};

mod context;

use liveletters_comment as comment;
use liveletters_config as config;
use liveletters_cu as cu;
use liveletters_doctor as doctor;
use liveletters_feed as feed;
use liveletters_inbox as inbox;
use liveletters_init as init;
use liveletters_lltt_sync as lltt_sync;
use liveletters_log as log;
use liveletters_outbox as outbox;
use liveletters_output::CommandContext;
use liveletters_post as post;
use liveletters_settings as settings;
use liveletters_status as status;
use liveletters_sub as sub;
use liveletters_thread as thread;

#[derive(Debug, Parser)]
#[command(name = "lltt", version, about = "LiveLetters CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Инициализировать домашний каталог.
    Init(init::Args),
    /// Управление текущим пользователем liveletters.
    Cu(cu::Args),
    /// Управление пользователями liveletters.
    User(cu::Args),
    /// Управление подписками на блоги.
    Sub(sub::Args),
    /// Показать ленту подписок.
    Feed(feed::Args),
    /// Управление входящей почтой.
    Inbox(inbox::Args),
    /// Создать пост.
    Post(post::Args),
    /// Создать комментарий.
    Comment(comment::Args),
    /// Показать исходящую очередь.
    Outbox(outbox::Args),
    /// Показать тред поста.
    Thread(thread::Args),
    /// Краткий отчёт о состоянии.
    Status(status::Args),
    /// Диагностика состояния.
    Doctor(doctor::Args),
    /// Показать или изменить настройки.
    Settings(settings::Args),
    /// Изменить одну настройку (сокращение для `lltt settings set <ключ> <значение>`).
    Set(SetArgs),
    /// Сетевая синхронизация.
    Sync(lltt_sync::Args),
}

#[derive(Debug, clap::Args)]
struct SetArgs {
    /// Имя настройки (например, `log.level`, `smtp.host`).
    key: String,
    /// Новое значение.
    value: String,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let mode = context_mode_for(&cli.command);
    let need_existing_home = !matches!(mode, context::ContextMode::Init);

    let ctx = match build_context_for(mode) {
        Ok(ctx) => ctx,
        Err(error) => {
            eprintln!("ошибка: {error}");
            return ExitCode::from(2);
        }
    };

    if need_existing_home && !ctx.home.exists() {
        eprintln!(
            "ошибка: домашний каталог не существует: {}",
            ctx.home.display()
        );
        eprintln!("запустите `lltt init` для инициализации");
        return ExitCode::from(2);
    }

    let result = match cli.command {
        Command::Init(args) => {
            let r = init::run(&ctx, &args);
            init_logger(&ctx);
            r
        }
        Command::Cu(args) => {
            init_logger(&ctx);
            cu::run_current(&ctx, &args)
        }
        Command::User(args) => {
            init_logger(&ctx);
            cu::run_user(&ctx, &args)
        }
        Command::Sub(args) => {
            init_logger(&ctx);
            sub::run(&ctx, &args)
        }
        Command::Feed(args) => {
            init_logger(&ctx);
            feed::run(&ctx, &args)
        }
        Command::Inbox(args) => {
            init_logger(&ctx);
            inbox::run(&ctx, &args)
        }
        Command::Post(args) => {
            init_logger(&ctx);
            post::run(&ctx, &args)
        }
        Command::Comment(args) => {
            init_logger(&ctx);
            comment::run(&ctx, &args)
        }
        Command::Outbox(args) => {
            init_logger(&ctx);
            outbox::run(&ctx, &args)
        }
        Command::Thread(args) => {
            init_logger(&ctx);
            thread::run(&ctx, &args)
        }
        Command::Status(args) => {
            init_logger(&ctx);
            status::run(&ctx, &args)
        }
        Command::Doctor(args) => {
            init_logger(&ctx);
            doctor::run(&ctx, &args)
        }
        Command::Settings(args) => {
            init_logger(&ctx);
            settings::run(&ctx, &args)
        }
        Command::Set(args) => {
            init_logger(&ctx);
            settings::set::run_directly(
                &ctx.home,
                &ctx.state_home,
                &ctx.identity_name,
                &args.key,
                &args.value,
            )
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        }
        Command::Sync(args) => {
            init_logger(&ctx);
            lltt_sync::run(&ctx, &args)
        }
    };

    log::shutdown();

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("ошибка: {error}");
            ExitCode::from(1)
        }
    }
}

/// Инициализирует глобальный логгер, если в `GlobalConfig` указан
/// уровень, отличный от `off`. Ошибки записи в файл журнала не мешают
/// работе CLI — только пишутся в stderr.
fn init_logger(ctx: &CommandContext) {
    let log = config::load_global(&ctx.home)
        .map(|cfg| cfg.log)
        .unwrap_or_default();
    if let Err(error) = log::init(&ctx.home, &log) {
        eprintln!("предупреждение: не удалось инициализировать журнал: {error}");
    }
}

fn context_mode_for(command: &Command) -> context::ContextMode {
    match command {
        Command::Init(_) => context::ContextMode::Init,
        Command::Cu(args) => {
            if cu_requires_current(&args.tokens) {
                context::ContextMode::RequiresCurrent
            } else {
                context::ContextMode::AllowMissingCurrent
            }
        }
        Command::User(_) => context::ContextMode::AllowMissingCurrent,
        _ => context::ContextMode::RequiresCurrent,
    }
}

fn cu_requires_current(tokens: &[String]) -> bool {
    match tokens {
        [] => true,
        [one] if one == "show" => true,
        [one, flag] if one == "show" && flag == "--reveal" => true,
        [one] if one == "posts" => true,
        [one, flag, _] if one == "posts" && flag == "--limit" => true,
        [one, flag] if one == "posts" && flag.starts_with("--limit=") => true,
        _ => false,
    }
}

fn build_context_for(mode: context::ContextMode) -> Result<CommandContext, context::ContextError> {
    // CLI-аргумента --as для текущего пользователя
    // не предусмотрено; имя читается ТОЛЬКО из `<home>/current-user`.
    context::build_context(mode)
}
