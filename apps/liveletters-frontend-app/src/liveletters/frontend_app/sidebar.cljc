(ns liveletters.frontend-app.sidebar
  "Sidebar: кнопки Home, Feed, список подписок."
  (:require [liveletters.ui-kit.icons :as icons]))

(def fake-subscriptions
  [{:label "alice@example.com"
    :meta "Личный блог"
    :accent "#6ab3ea"}
   {:label "bob@dev.local"
    :meta "Технологии и заметки"
    :accent "#79c29d"}
   {:label "news@tech.io"
    :meta "Канал обновлений"
    :accent "#8f97f7"}])

(defn- shell-item [{:keys [label icon active? on-click]}]
  [:button {:type "button"
            :class "ll-sidebar-item"
            :style {:display "flex"
                    :align-items "center"
                    :gap "11px"
                    :width "calc(100% - 16px)"
                    :padding "10px 12px"
                    :margin "0 8px 4px"
                    :background (if active?
                                  "rgba(255,255,255,0.055)"
                                  "transparent")
                    :color (if active?
                             "var(--text-primary)"
                             "var(--text-secondary)")
                    :border "1px solid transparent"
                    :border-radius "12px"
                    :cursor "pointer"
                    :font-size "14px"
                    :font-weight (if active? 600 500)
                    :transition "background 120ms, color 120ms, border-color 120ms"
                    :text-align "left"}
            :on-click on-click}
   [:span {:style {:display "inline-flex"
                   :align-items "center"
                   :justify-content "center"
                   :width "18px"
                   :opacity (if active? 0.95 0.7)}}
    icon]
   [:span label]])

(defn- subscription-item [{:keys [label meta accent]}]
  [:button {:type "button"
            :style {:display "grid"
                    :grid-template-columns "34px minmax(0, 1fr)"
                    :gap "10px"
                    :align-items "center"
                    :width "calc(100% - 12px)"
                    :padding "9px 10px"
                    :margin "0 6px 2px"
                    :background "transparent"
                    :border "1px solid transparent"
                    :border-radius "14px"
                    :cursor "pointer"
                    :text-align "left"
                    :transition "background 120ms, border-color 120ms"}}
   [:span {:style {:display "inline-flex"
                   :align-items "center"
                   :justify-content "center"
                   :width "34px"
                   :height "34px"
                   :border-radius "999px"
                   :background (str "linear-gradient(135deg, " accent ", rgba(255,255,255,0.18))")
                   :color "#f8fbff"
                   :font-size "13px"
                   :font-weight 700
                   :box-shadow "inset 0 1px 0 rgba(255,255,255,0.16)"}}
    (.toUpperCase (subs label 0 1))]
   [:span {:style {:display "grid"
                   :gap "2px"
                   :min-width 0}}
    [:span {:style {:font-size "13px"
                    :font-weight 500
                    :color "var(--text-primary)"
                    :white-space "nowrap"
                    :overflow "hidden"
                    :text-overflow "ellipsis"}}
     label]
    [:span {:style {:font-size "12px"
                    :color "var(--text-tertiary)"
                    :white-space "nowrap"
                    :overflow "hidden"
                    :text-overflow "ellipsis"}}
     meta]]])

(defn sidebar-view [{:keys [active-page on-home on-feed on-settings]}]
  [:aside {:class "ll-sidebar"}
   [:div {:style {:display "grid"
                  :gap "4px"
                  :padding "10px 0 6px"}}
    [shell-item {:label "Home"
                 :icon (icons/icon-home)
                 :active? (= active-page :home)
                 :on-click on-home}]
    [shell-item {:label "Feed"
                 :icon (icons/icon-rss)
                 :active? (= active-page :feed)
                 :on-click on-feed}]]
   [:div {:style {:height "1px"
                  :background "var(--border-soft)"
                  :margin "8px 12px 10px"}}]
   [:div {:style {:display "flex"
                  :align-items "center"
                  :justify-content "space-between"
                  :padding "0 16px 8px"}}
    [:span {:style {:font-size "11px"
                    :font-weight 700
                    :letter-spacing "0.12em"
                    :text-transform "uppercase"
                    :color "var(--text-tertiary)"}}
     "Подписки"]
    [:button {:type "button"
              :on-click on-settings
              :style {:display "inline-flex"
                      :align-items "center"
                      :justify-content "center"
                      :width "24px"
                      :height "24px"
                      :padding 0
                      :border "none"
                      :border-radius "999px"
                      :background "transparent"
                      :color "var(--text-tertiary)"
                      :cursor "pointer"}}
     (icons/icon-settings)]]
   [:div {:style {:display "grid"
                  :gap "1px"}}
    (for [subscription fake-subscriptions]
      ^{:key (:label subscription)}
      [subscription-item subscription])]
   [:div {:style {:flex 1}}]])
