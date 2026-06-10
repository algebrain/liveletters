//! `lltt sync push` — отправить исходящие записи из `outbox` через SMTP
//! по адресации, заданной самой записью; при успехе — удалить запись из `outbox`.

use liveletters_mail::{
    ConfiguredSmtpTransport, OutgoingEmail, SendStatus, SmtpTransportConfig, TransportError,
    build_protocol_email,
};
use liveletters_output::CommandContext;
use liveletters_protocol::{ProtocolMessage, decode_message};
use liveletters_store::{OutboxDelivery, OutboxRecord, Store};

use crate::error::SyncError;
use crate::pull::parse_security;

pub fn run(ctx: &CommandContext) -> Result<(), SyncError> {
    let store = Store::open_for_home_dir(&ctx.state_home)?;
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
        match send_outbox_record(&store, &transport, &mail.smtp_username, record) {
            Ok(n) if n > 0 => {
                store.delete_outbox_record(&record.event_id)?;
                sent += n;
                liveletters_log::log_info(format!(
                    "sync.push event_id={} resource_id={} sent={}",
                    record.event_id, record.resource_id, n,
                ));
            }
            Ok(_) => {
                eprintln!(
                    "предупреждение: нет адресатов для {}, outbox-запись {} оставлена",
                    record.resource_id, record.event_id
                );
                liveletters_log::log_warn(format!(
                    "sync.push event_id={} resource_id={} recipients=0",
                    record.event_id, record.resource_id,
                ));
            }
            Err(error) => {
                eprintln!("ошибка отправки {}: {error}", record.event_id);
                liveletters_log::log_error(format!(
                    "sync.push event_id={} resource_id={} error={error:?}",
                    record.event_id, record.resource_id,
                ));
                failed += 1;
            }
        }
    }

    println!("подключено к {}", mail.smtp_host);
    println!("отправлено писем:     {sent}");
    println!("ошибок отправки:      {failed}");
    Ok(())
}

pub fn send_outbox_record(
    store: &Store,
    transport: &ConfiguredSmtpTransport,
    from: &str,
    record: &OutboxRecord,
) -> Result<usize, SyncError> {
    let message = decode_message(&record.message_body)
        .map_err(|_| SyncError::OutboxDecode(record.event_id.clone()))?;

    let recipients = resolve_recipients(store, record)?;
    if recipients.is_empty() {
        return Ok(0);
    }

    let mut count = 0;
    for addr in &recipients {
        send_one(transport, from, addr, record, &message)
            .map_err(|e| SyncError::Smtp(format!("{e:?}")))?;
        count += 1;
    }
    Ok(count)
}

fn resolve_recipients(store: &Store, record: &OutboxRecord) -> Result<Vec<String>, SyncError> {
    match &record.delivery {
        OutboxDelivery::Direct(addrs) => Ok(addrs.clone()),
        OutboxDelivery::ResourceSubscribers => Ok(store
            .list_subscriptions_for_resource(&record.resource_id)?
            .into_iter()
            .map(|sub| sub.subscriber_delivery_address)
            .collect()),
    }
}

fn send_one(
    transport: &ConfiguredSmtpTransport,
    from: &str,
    to: &str,
    record: &OutboxRecord,
    message: &ProtocolMessage,
) -> Result<SendStatus, TransportError> {
    let outgoing: OutgoingEmail = build_protocol_email(from, to, &record.event_type, message)?;
    transport.send(&outgoing)
}

pub fn default_profile_id(name: &str) -> String {
    if name.is_empty() {
        "default".to_owned()
    } else {
        name.to_owned()
    }
}
