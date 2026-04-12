(ns liveletters.frontend-app.content-views
  "Компоненты для трёх стилей основной области."
  (:require [liveletters.frontend-app.theme.content-feed :as feed]
            [liveletters.frontend-app.theme.content-thread :as thread]
            [liveletters.frontend-app.theme.content-editor :as editor]
            [liveletters.ui-kit.icons :as icons]))

;; ---------- Лента постов ----------

(defn- fake-post [author time body]
  [feed/post-card {}
   [feed/post-card-header {}
    [feed/post-card-author {} author]
    [feed/post-card-time {} time]]
   [feed/post-card-body {} body]])

(defn feed-page []
  [feed/feed-container {}
   [fake-post "alice@example.com" "2 часа назад"
    "Первый тестовый пост. Здесь будет текст записи, который показывается в ленте подписок."]
   [fake-post "bob@dev.local" "5 часов назад"
    "Второй пост про разработку и технологии. Markdown-рендеринг будет добавлен позже."]
   [fake-post "news@tech.io" "вчера"
    "Третий пост — пример длинного текста в ленте. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua."]])

;; ---------- Пост с обсуждением ----------

(defn- fake-comment [author body]
  [thread/comment-item {}
   [:div {}
    [thread/comment-author {} author]
    [thread/comment-body {} body]]])

(defn post-thread-page []
  [thread/thread-container {}
   ;; Пост
   [thread/thread-post {}
    [thread/thread-post-title {} "Заголовок тестового поста"]
    [thread/thread-post-meta {} "alice@example.com · 2 часа назад"]
    [thread/thread-post-body {}
     "Основной текст поста. Здесь может быть Markdown с форматированием, ссылками и прочим. Пока показываем как plain text."]]
   ;; Комментарии
   [:div {:style {:font-size "14px" :color "var(--text-secondary)" :margin-bottom "8px"}}
    "2 комментария"]
   [thread/comment-list {}
    [fake-comment "bob@dev.local" "Отличный пост, полностью согласен!"]
    [fake-comment "news@tech.io" "Интересная точка зрения, но есть нюанс с производительностью."]]
   ;; Ввод комментария
   [thread/comment-input-area {}
    [:div {:style {:margin-bottom "8px" :font-size "12px" :color "var(--text-secondary)"}}
     "Написать комментарий"]
    [thread/comment-textarea {:placeholder "Ваш комментарий..."}]
    [:button {:type "button"
              :style {:padding "8px 18px"
                      :border-radius "8px"
                      :background "var(--accent)"
                      :color "#fff"
                      :border "none"
                      :font-size "13px"
                      :font-weight 600
                      :cursor "pointer"}}
     "Отправить"]]])

;; ---------- Редактор поста ----------

(defn editor-page []
  [editor/editor-container {}
   [editor/editor-layout {}
    [editor/editor-pane {}
     [editor/editor-textarea
      {:default-value (str "## Короткое объявление\n\n"
                           "Пробую более спокойный экран редактора.\n\n"
                           "Здесь должно быть меньше обвязки и меньше тяжёлых панелей, "
                           "чтобы внимание оставалось на тексте.\n\n"
                           "- простой черновик\n"
                           "- компактный preview\n"
                           "- публикация сразу видна внизу\n\n"
                           "Если направление верное, следующим шагом можно уже подключать "
                           "реальную форму создания поста.")
       :placeholder "Введите текст поста в формате Markdown..."}]]
    [editor/editor-pane {}
     [editor/preview-pane {}
      [editor/preview-title {} "Короткое объявление"]
      [editor/preview-meta {} "alice@example.com · несколько секунд назад · public"]
      [editor/preview-body {}
       [:p {:style {:margin 0}}
        "Пробую более спокойный экран редактора."]
       [:p {:style {:margin 0}}
        "В этом варианте меньше декоративной вложенности и меньше поверхностей, которые спорят с текстом."]
       [:ul {:style {:margin 0 :padding-left "18px" :color "var(--text-secondary)"}}
        [:li {} "черновик слева"]
        [:li {} "preview справа"]
        [:li {} "кнопки сразу видны внизу"]]]
      [editor/preview-note {}
       "Markdown-рендеринг и реальные backend-связи можно подключить следующим этапом, не раздувая экран дополнительными панелями."]]]]
   [editor/editor-actions {}
    [editor/secondary-button {:type "button"}
     "Отмена"]
    [editor/publish-button {} "Опубликовать"]]])
