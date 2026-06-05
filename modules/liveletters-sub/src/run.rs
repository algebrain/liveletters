use liveletters_config::{IdentityConfig, save_identity};
use liveletters_store::Store;

use crate::{Args, SubError, args::SubAction};

pub fn subscribe(
    ctx: &liveletters_output::CommandContext,
    store: &Store,
    resource_address: &str,
) -> Result<(), SubError> {
    let mut identity = load_current_identity(ctx)?;
    let delivery_address = derive_delivery_address(&identity);

    let created_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let core = liveletters_app_core::AppCore::new(store);
    let _ = core.subscribe(liveletters_app_core::SubscribeCommand {
        resource_address,
        subscriber_account_id: identity.account_id(),
        subscriber_delivery_address: &delivery_address,
        created_at,
    })?;

    add_to_local_subscriptions(&mut identity, resource_address)?;
    save_identity(&ctx.home, &ctx.identity_name, &identity)?;

    println!(
        "подписан на {}: посты будут приходить на {}",
        resource_address, delivery_address
    );
    Ok(())
}

pub fn unsubscribe(
    ctx: &liveletters_output::CommandContext,
    store: &Store,
    resource_address: &str,
) -> Result<(), SubError> {
    let mut identity = load_current_identity(ctx)?;
    let created_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let core = liveletters_app_core::AppCore::new(store);
    let _ = core.unsubscribe(liveletters_app_core::UnsubscribeCommand {
        resource_address,
        subscriber_account_id: identity.account_id(),
        created_at,
    })?;

    remove_from_local_subscriptions(&mut identity, resource_address);
    save_identity(&ctx.home, &ctx.identity_name, &identity)?;

    println!("отписан от {}", resource_address);
    Ok(())
}

pub fn list_subscriptions(
    ctx: &liveletters_output::CommandContext,
    store: &Store,
) -> Result<(), SubError> {
    let identity = load_current_identity(ctx)?;
    let subscribed: Vec<String> = identity
        .subscriptions()
        .iter()
        .map(|a| a.as_str().to_owned())
        .collect();
    let core = liveletters_app_core::AppCore::new(store);
    let list = core.list_subscriptions(liveletters_app_core::ListSubscriptionsQuery {
        owned_resource_address: identity.mail().publish(),
        subscribed_addresses: &subscribed,
    })?;

    println!("подписан на:");
    if list.subscribed_addresses().is_empty() {
        println!("  (пусто)");
    } else {
        for addr in list.subscribed_addresses() {
            println!("  {}", addr);
        }
    }

    println!("мои подписчики:");
    if list.owned_subscribers().is_empty() {
        println!("  (пусто)");
    } else {
        for sub in list.owned_subscribers() {
            println!(
                "  {}  →  {}",
                sub.subscriber_account_id, sub.subscriber_delivery_address
            );
        }
    }
    Ok(())
}

fn load_current_identity(
    ctx: &liveletters_output::CommandContext,
) -> Result<IdentityConfig, SubError> {
    Ok(liveletters_config::load_identity(
        &ctx.home,
        &ctx.identity_name,
    )?)
}

fn derive_delivery_address(identity: &IdentityConfig) -> String {
    identity
        .mail()
        .receive()
        .first()
        .map(String::as_str)
        .unwrap_or(identity.mail().publish())
        .to_owned()
}

fn add_to_local_subscriptions(
    identity: &mut IdentityConfig,
    address: &str,
) -> Result<(), SubError> {
    let new_address = liveletters_domain::ResourceAddress::new(address)?;
    if !identity.subscriptions().iter().any(|a| a == &new_address) {
        let mut updated = identity.subscriptions().to_vec();
        updated.push(new_address);
        identity.meta.subscriptions = updated;
    }
    Ok(())
}

fn remove_from_local_subscriptions(identity: &mut IdentityConfig, address: &str) {
    if let Ok(target) = liveletters_domain::ResourceAddress::new(address) {
        identity.meta.subscriptions.retain(|a| a != &target);
    }
}

pub fn ensure_subscription_address_valid(address: &str) -> Result<(), SubError> {
    liveletters_domain::ResourceAddress::new(address)?;
    Ok(())
}

pub fn run(ctx: &liveletters_output::CommandContext, args: &Args) -> Result<(), SubError> {
    let action = parse_action(&args.tokens)?;
    let store = liveletters_store::Store::open_for_home_dir(&ctx.home)?;
    match action {
        SubAction::Subscribe { resource_address } => subscribe(ctx, &store, &resource_address)?,
        SubAction::Rm { resource_address } => unsubscribe(ctx, &store, &resource_address)?,
        SubAction::List => list_subscriptions(ctx, &store)?,
    }
    Ok(())
}

fn parse_action(tokens: &[String]) -> Result<SubAction, SubError> {
    match tokens.first().map(String::as_str) {
        None => Err(SubError::InvalidArgs(
            "`lltt sub` требует адрес блога".to_owned(),
        )),
        Some("list") => {
            if tokens.len() != 1 {
                return Err(SubError::InvalidArgs(format!(
                    "`list` не принимает аргументов, получили: {tokens:?}"
                )));
            }
            Ok(SubAction::List)
        }
        Some("rm") => {
            if tokens.len() != 2 {
                return Err(SubError::InvalidArgs(format!(
                    "`rm` требует один адрес, получили: {tokens:?}"
                )));
            }
            ensure_subscription_address_valid(&tokens[1])?;
            Ok(SubAction::Rm {
                resource_address: tokens[1].clone(),
            })
        }
        Some(other) => {
            if tokens.len() != 1 {
                return Err(SubError::InvalidArgs(format!(
                    "лишние аргументы: {tokens:?}"
                )));
            }
            ensure_subscription_address_valid(other)?;
            Ok(SubAction::Subscribe {
                resource_address: other.to_owned(),
            })
        }
    }
}
