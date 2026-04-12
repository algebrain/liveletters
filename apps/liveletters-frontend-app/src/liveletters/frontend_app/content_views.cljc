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
   [feed/feed-header {} "Home feed"]
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
   [editor/editor-header {} "Новый пост"]
   [editor/editor-layout {}
    ;; Левая панель — редактор
    [editor/editor-pane {}
     [editor/editor-pane-label {} "Редактор"]
     [editor/editor-textarea {:placeholder "Введите текст поста в формате Markdown..."}]]
    ;; Правая панель — предпросмотр
    [editor/editor-pane {}
     [editor/editor-pane-label {} "Предпросмотр"]
     [editor/preview-pane {}
      [:div {:style {:color "var(--text-secondary)" :font-style "italic"}}
       "Здесь будет предпросмотр Markdown..."]]]]
   ;; Кнопки
   [editor/editor-actions {}
    [:button {:type "button"
              :style {:padding "10px 18px"
                      :border-radius "10px"
                      :background "transparent"
                      :color "var(--text-secondary)"
                      :border "1px solid rgba(255,255,255,0.08)"
                      :font-size "14px"
                      :cursor "pointer"}}
     "Отмена"]
    [editor/publish-button {} "Опубликовать"]]])
