use std::error::Error;

use crate::{Args, CommandContext, SyncAction};

pub fn run(ctx: &CommandContext, args: &Args) -> Result<(), Box<dyn Error + Send + Sync>> {
    match args.action {
        SyncAction::Pull => pull_dispatch(ctx),
        SyncAction::Push => push_dispatch(ctx),
    }
}

#[cfg(feature = "network")]
fn pull_dispatch(ctx: &CommandContext) -> Result<(), Box<dyn Error + Send + Sync>> {
    crate::pull::run(ctx).map_err(|e| Box::new(e) as _)
}

#[cfg(not(feature = "network"))]
fn pull_dispatch(_ctx: &CommandContext) -> Result<(), Box<dyn Error + Send + Sync>> {
    Err(Box::new(NetworkFeatureDisabled) as _)
}

#[cfg(feature = "network")]
fn push_dispatch(ctx: &CommandContext) -> Result<(), Box<dyn Error + Send + Sync>> {
    crate::push::run(ctx).map_err(|e| Box::new(e) as _)
}

#[cfg(not(feature = "network"))]
fn push_dispatch(_ctx: &CommandContext) -> Result<(), Box<dyn Error + Send + Sync>> {
    Err(Box::new(NetworkFeatureDisabled) as _)
}

#[cfg(not(feature = "network"))]
#[derive(Debug)]
pub struct NetworkFeatureDisabled;

#[cfg(not(feature = "network"))]
impl std::fmt::Display for NetworkFeatureDisabled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "подкоманды sync pull/push требуют сборки lltt с признаком network"
        )
    }
}

#[cfg(not(feature = "network"))]
impl Error for NetworkFeatureDisabled {}
