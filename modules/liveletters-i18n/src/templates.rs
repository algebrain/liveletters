use crate::Locale;

pub(super) fn template(key: &str, locale: Locale) -> Option<&'static str> {
    let (ru, en): (&'static str, &'static str) = match key {
        "post_created.subject" => (
            "Новая запись в журнале %resource%",
            "New post in journal %resource%",
        ),
        "post_created.body" => (
            "Новая запись в журнале %resource%:\n\n%body%\n\n— LiveLetters",
            "New post in journal %resource%:\n\n%body%\n\n— LiveLetters",
        ),
        "comment_created.subject" => ("Новый комментарий от %sender%", "New comment by %sender%"),
        "comment_created.body" => (
            "%sender% оставил(а) комментарий к записи %post_id%:\n\n%body%\n\n— LiveLetters",
            "%sender% has commented on post %post_id%:\n\n%body%\n\n— LiveLetters",
        ),
        "comment_created_redistribute.subject" => (
            "Новый комментарий в %resource%",
            "New comment in %resource%",
        ),
        "comment_created_redistribute.body" => (
            "%sender% оставил(а) комментарий к записи %post_id%:\n\n%body%\n\n— LiveLetters",
            "%sender% has commented on post %post_id%:\n\n%body%\n\n— LiveLetters",
        ),
        "comment_edited.subject" => ("Комментарий изменён: %sender%", "Comment edited: %sender%"),
        "comment_edited.body" => (
            "%sender% отредактировал(а) комментарий к записи %post_id%:\n\n%body%\n\n— LiveLetters",
            "%sender% has edited a comment on post %post_id%:\n\n%body%\n\n— LiveLetters",
        ),
        "post_hidden.subject" => ("Запись скрыта: %actor%", "Post hidden: %actor%"),
        "post_hidden.body" => (
            "%actor% скрыл(а) запись %post_id% в вашем блоге.\n\n— LiveLetters",
            "%actor% has hidden post %post_id% in your blog.\n\n— LiveLetters",
        ),
        "subscription_requested.subject" => {
            ("Подписка: %subscriber%", "New subscription: %subscriber%")
        }
        "subscription_requested.body" => (
            "%subscriber% подписался(ась) на вас в LiveLetters (блог %resource%).\n\n— LiveLetters",
            "%subscriber% has subscribed to you in LiveLetters (blog %resource%).\n\n— LiveLetters",
        ),
        "subscription_confirmed_accepted.subject" => (
            "Подписка подтверждена: %resource%",
            "Subscription confirmed: %resource%",
        ),
        "subscription_confirmed_accepted.body" => (
            "%owner% подтвердил(а) вашу подписку на %resource% в LiveLetters.\n\n— LiveLetters",
            "%owner% confirmed your subscription to %resource% in LiveLetters.\n\n— LiveLetters",
        ),
        "subscription_confirmed_declined.subject" => (
            "Запрос на подписку отклонён",
            "Subscription request declined",
        ),
        "subscription_confirmed_declined.body" => (
            "%owner% отклонил(а) ваш запрос на подписку на %resource% в LiveLetters.\n\n— LiveLetters",
            "%owner% declined your subscription request to %resource% in LiveLetters.\n\n— LiveLetters",
        ),
        "subscription_revoked.subject" => ("Отписка: %subscriber%", "Unsubscribed: %subscriber%"),
        "subscription_revoked.body" => (
            "%subscriber% отписался(ась) от вас в LiveLetters (блог %resource%).\n\n— LiveLetters",
            "%subscriber% has unsubscribed from you in LiveLetters (blog %resource%).\n\n— LiveLetters",
        ),
        _ => return None,
    };
    Some(if matches!(locale, Locale::Ru) { ru } else { en })
}
