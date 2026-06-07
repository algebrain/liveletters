use liveletters_i18n::{Locale, Vars, detect_system_locale, parse_locale, translate};
use liveletters_store::UserSettingsRecord;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubjectAndBody {
    pub subject: String,
    pub body: String,
}

/// Читает язык из записи пользователя; при отсутствии записи или
/// неподдерживаемом значении возвращает локаль, определённую по
/// переменным окружения (`LC_ALL`/`LC_MESSAGES`/`LANG`), с fallback `En`.
pub fn locale_for(record: Option<&UserSettingsRecord>) -> Locale {
    record
        .and_then(|r| parse_locale(&r.language).ok())
        .unwrap_or_else(detect_system_locale)
}

pub fn post_created(
    record: Option<&UserSettingsRecord>,
    sender: &str,
    resource: &str,
    body: &str,
) -> SubjectAndBody {
    let loc = locale_for(record);
    let vars = Vars(&[("sender", sender), ("resource", resource), ("body", body)]);
    SubjectAndBody {
        subject: translate("post_created.subject", loc, vars)
            .expect("шаблон post_created.subject присутствует в таблице"),
        body: translate("post_created.body", loc, vars)
            .expect("шаблон post_created.body присутствует в таблице"),
    }
}

pub fn comment_created(
    record: Option<&UserSettingsRecord>,
    sender: &str,
    post_id: &str,
    body: &str,
) -> SubjectAndBody {
    let loc = locale_for(record);
    let vars = Vars(&[("sender", sender), ("post_id", post_id), ("body", body)]);
    SubjectAndBody {
        subject: translate("comment_created.subject", loc, vars)
            .expect("шаблон comment_created.subject присутствует в таблице"),
        body: translate("comment_created.body", loc, vars)
            .expect("шаблон comment_created.body присутствует в таблице"),
    }
}

pub fn comment_edited(
    record: Option<&UserSettingsRecord>,
    sender: &str,
    post_id: &str,
    body: &str,
) -> SubjectAndBody {
    let loc = locale_for(record);
    let vars = Vars(&[("sender", sender), ("post_id", post_id), ("body", body)]);
    SubjectAndBody {
        subject: translate("comment_edited.subject", loc, vars)
            .expect("шаблон comment_edited.subject присутствует в таблице"),
        body: translate("comment_edited.body", loc, vars)
            .expect("шаблон comment_edited.body присутствует в таблице"),
    }
}

pub fn post_hidden(
    record: Option<&UserSettingsRecord>,
    actor: &str,
    post_id: &str,
) -> SubjectAndBody {
    let loc = locale_for(record);
    let vars = Vars(&[("actor", actor), ("post_id", post_id)]);
    SubjectAndBody {
        subject: translate("post_hidden.subject", loc, vars)
            .expect("шаблон post_hidden.subject присутствует в таблице"),
        body: translate("post_hidden.body", loc, vars)
            .expect("шаблон post_hidden.body присутствует в таблице"),
    }
}

pub fn subscription_changed_active(
    record: Option<&UserSettingsRecord>,
    subscriber: &str,
    resource: &str,
) -> SubjectAndBody {
    let loc = locale_for(record);
    let vars = Vars(&[("subscriber", subscriber), ("resource", resource)]);
    SubjectAndBody {
        subject: translate("subscription_changed.active.subject", loc, vars)
            .expect("шаблон subscription_changed.active.subject присутствует в таблице"),
        body: translate("subscription_changed.active.body", loc, vars)
            .expect("шаблон subscription_changed.active.body присутствует в таблице"),
    }
}

pub fn subscription_changed_inactive(
    record: Option<&UserSettingsRecord>,
    subscriber: &str,
    resource: &str,
) -> SubjectAndBody {
    let loc = locale_for(record);
    let vars = Vars(&[("subscriber", subscriber), ("resource", resource)]);
    SubjectAndBody {
        subject: translate("subscription_changed.inactive.subject", loc, vars)
            .expect("шаблон subscription_changed.inactive.subject присутствует в таблице"),
        body: translate("subscription_changed.inactive.body", loc, vars)
            .expect("шаблон subscription_changed.inactive.body присутствует в таблице"),
    }
}
