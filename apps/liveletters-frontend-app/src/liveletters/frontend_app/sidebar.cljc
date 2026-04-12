(ns liveletters.frontend-app.sidebar
  "Sidebar: кнопки Home, Feed, список подписок."
  (:require [liveletters.ui-kit.icons :as icons]))

(defn- nav-item [{:keys [label icon active? on-click]}]
  [:button {:type "button"
            :class "ll-sidebar-item"
            :style {:display "flex"
                    :align-items "center"
                    :gap "12px"
                    :width "100%"
                    :padding "10px 14px"
                    :background (if active? "var(--bg-tertiary)" "transparent")
                    :color "var(--text-primary)"
                    :border "none"
                    :border-radius "8px"
                    :cursor "pointer"
                    :font-size "14px"
                    :font-weight (if active? "600" "400")
                    :transition "background 120ms"
                    :margin "2px 8px"
                    :text-align "left"}
            :on-click on-click}
   (when icon [:span {:style {:display "flex" :opacity 0.7}} icon])
   [:span label]])

(defn sidebar-view [{:keys [active-page on-home on-feed on-settings]}]
  [:aside {:class "ll-sidebar"}
   ;; Home
   [nav-item {:label "Home"
              :icon (icons/icon-home)
              :active? (= active-page :home)
              :on-click on-home}]
   ;; Feed
   [nav-item {:label "Feed"
              :icon (icons/icon-rss)
              :active? (= active-page :feed)
              :on-click on-feed}]
   ;; Разделитель
   [:div {:style {:height "1px"
                  :background "rgba(255,255,255,0.06)"
                  :margin "8px 14px"}}]
   ;; Заглушка подписок
   [:div {:style {:padding "8px 14px"
                  :font-size "12px"
                  :color "var(--text-secondary)"
                  :text-transform "uppercase"
                  :letter-spacing "0.08em"}}
    "Подписки"]
   ;; Фейковые подписки для визуальной проверки
   [nav-item {:label "alice@example.com"}]
   [nav-item {:label "bob@dev.local"}]
   [nav-item {:label "news@tech.io"}]
   ;; Нижняя часть — пусто (кнопка «Написать пост» в навбаре)
   [:div {:style {:flex 1}}]])
