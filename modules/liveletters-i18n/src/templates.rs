use crate::Locale;

pub(super) fn template(key: &str, locale: Locale) -> Option<&'static str> {
    let (ru, en): (&'static str, &'static str) = match key {
        "post_created.subject" => (
            "Новая запись от %sender% в %resource%",
            "New post by %sender% in %resource%",
        ),
        "post_created.body" => (
            "%sender% написал(а) новую запись в %resource%:\n\n%body%\n\n— LiveLetters",
            "%sender% has created a new post in %resource%:\n\n%body%\n\n— LiveLetters",
        ),
        "comment_created.subject" => ("Новый комментарий от %sender%", "New comment by %sender%"),
        "comment_created.body" => (
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
        "subscription_changed.active.subject" => {
            ("Подписка: %subscriber%", "New subscription: %subscriber%")
        }
        "subscription_changed.active.body" => (
            "%subscriber% подписался(ась) на вас в LiveLetters (блог %resource%).\n\n— LiveLetters",
            "%subscriber% has subscribed to you in LiveLetters (blog %resource%).\n\n— LiveLetters",
        ),
        "subscription_changed.inactive.subject" => {
            ("Отписка: %subscriber%", "Unsubscribed: %subscriber%")
        }
        "subscription_changed.inactive.body" => (
            "%subscriber% отписался(ась) от вас в LiveLetters (блог %resource%).\n\n— LiveLetters",
            "%subscriber% has unsubscribed from you in LiveLetters (blog %resource%).\n\n— LiveLetters",
        ),
        _ => return None,
    };
    Some(if matches!(locale, Locale::Ru) { ru } else { en })
}
