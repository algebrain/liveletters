//! `lltt sync push` — отправить исходящие записи из `outbox` через SMTP
//! подписчикам ресурса; при успехе — удалить запись из `outbox`.

use liveletters_mail::{
    ConfiguredSmtpTransport, OutgoingEmail, SendStatus, SmtpTransportConfig, TransportError,
    build_protocol_email,
};
use liveletters_output::CommandContext;
use liveletters_protocol::{ProtocolMessage, decode_message};
use liveletters_store::{OutboxRecord, Store, SubscriptionRecord};

use crate::error::SyncError;
use crate::pull::parse_security;

pub fn run(ctx: &CommandContext) -> Result<(), SyncError> {
    let store = Store::open_for_home_dir(&ctx.home)?;
    let profile_id = default_profile_id(&ctx.identity_name);
    let mail = store
        .get_mail_settings_record(&profile_id)?
        .ok_or_else(|| SyncError::MailSettingsMissing(profile_id.clone()))?;

    let transport = ConfiguredSmtpTransport::new(SmtpTransportConfig::new(
        &mail.smtp_host,
        mail.smtp_port,
        &mail.smtp_hello_domain,
        parse_security(&mail.smtp_security)?,
        liveletters_mail::MailAuth::Password {
            username: mail.smtp_username.clone(),
            password: mail.smtp_password,
        },
    ));

    let records = store.list_outbox_records()?;
    let mut sent = 0_usize;
    let mut failed = 0_usize;

    for record in &records {
        match send_to_subscribers(&store, &transport, &mail.smtp_username, record) {
            Ok(n) if n > 0 => {
                store.delete_outbox_record(&record.event_id)?;
                sent += n;
            }
            Ok(_) => {
                eprintln!(
                    "предупреждение: нет подписчиков для {}, outbox-запись {} оставлена",
                    record.resource_id, record.event_id
                );
            }
            Err(error) => {
                eprintln!("ошибка отправки {}: {error}", record.event_id);
                failed += 1;
            }
        }
    }

    println!("подключено к {}", mail.smtp_host);
    println!("отправлено писем:     {sent}");
    println!("ошибок отправки:      {failed}");
    Ok(())
}

pub fn send_to_subscribers(
    store: &Store,
    transport: &ConfiguredSmtpTransport,
    from: &str,
    record: &OutboxRecord,
) -> Result<usize, SyncError> {
    let message = decode_message(&record.message_body)
        .map_err(|_| SyncError::OutboxDecode(record.event_id.clone()))?;

    let subscribers = store.list_subscriptions_for_resource(&record.resource_id)?;
    if subscribers.is_empty() {
        return Ok(0);
    }

    let mut count = 0;
    for sub in &subscribers {
        send_one(transport, from, sub, &message).map_err(|e| SyncError::Smtp(format!("{e:?}")))?;
        count += 1;
    }
    Ok(count)
}

fn send_one(
    transport: &ConfiguredSmtpTransport,
    from: &str,
    subscriber: &SubscriptionRecord,
    message: &ProtocolMessage,
) -> Result<SendStatus, TransportError> {
    let outgoing: OutgoingEmail = build_protocol_email(
        from,
        &subscriber.subscriber_delivery_address,
        message.envelope().event_type(),
        message,
    )?;
    transport.send(&outgoing)
}

fn default_profile_id(name: &str) -> String {
    if name.is_empty() {
        "default".to_owned()
    } else {
        name.to_owned()
    }
}
