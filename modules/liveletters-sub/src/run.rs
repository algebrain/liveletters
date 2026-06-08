use liveletters_store::Store;

use crate::{Args, SubError, args::SubAction};

pub fn subscribe(
    ctx: &liveletters_output::CommandContext,
    store: &Store,
    resource_address: &str,
) -> Result<(), SubError> {
    let delivery_address = delivery_address_for(store, &ctx.identity_name)?;

    let created_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let core = liveletters_app_core::AppCore::new(store);
    let _ = core.subscribe(liveletters_app_core::SubscribeCommand {
        profile_id: &ctx.identity_name,
        resource_address,
        subscriber_delivery_address: &delivery_address,
        created_at,
    })?;

    store.add_local_subscription(&ctx.identity_name, resource_address)?;
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
    let delivery_address = delivery_address_for(store, &ctx.identity_name)?;
    let created_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let core = liveletters_app_core::AppCore::new(store);
    let _ = core.unsubscribe(liveletters_app_core::UnsubscribeCommand {
        profile_id: &ctx.identity_name,
        resource_address,
        subscriber_delivery_address: &delivery_address,
        created_at,
    })?;

    store.remove_local_subscription(&ctx.identity_name, resource_address)?;
    println!("отписан от {}", resource_address);
    Ok(())
}

pub fn list_subscriptions(
    ctx: &liveletters_output::CommandContext,
    store: &Store,
) -> Result<(), SubError> {
    let publish = store
        .get_user_settings_record(&ctx.identity_name)?
        .map(|r| r.email_address)
        .unwrap_or_default();
    let subscribed: Vec<String> = store.list_local_subscriptions(&ctx.identity_name)?;
    let core = liveletters_app_core::AppCore::new(store);
    let list = core.list_subscriptions(liveletters_app_core::ListSubscriptionsQuery {
        owned_resource_address: &publish,
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
            println!("  {}", sub.subscriber_delivery_address);
        }
    }
    Ok(())
}

fn delivery_address_for(store: &Store, profile_id: &str) -> Result<String, SubError> {
    let receive = store.list_receive_addresses(profile_id)?;
    if let Some(addr) = receive.first() {
        return Ok(addr.clone());
    }
    let email = store
        .get_user_settings_record(profile_id)?
        .map(|r| r.email_address)
        .unwrap_or_default();
    Ok(email)
}

pub fn ensure_subscription_address_valid(address: &str) -> Result<(), SubError> {
    liveletters_domain::ResourceAddress::new(address)?;
    Ok(())
}

pub fn run(ctx: &liveletters_output::CommandContext, args: &Args) -> Result<(), SubError> {
    let action = parse_action(&args.tokens)?;
    let store = Store::open_for_home_dir(&ctx.state_home)?;
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
