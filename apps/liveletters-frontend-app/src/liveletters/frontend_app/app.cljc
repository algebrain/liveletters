(ns liveletters.frontend-app.app
  (:require [liveletters.frontend-app.pages :as pages]
            [liveletters.frontend-app.routes :as routes]
            [liveletters.frontend-app.selectors :as selectors]
            [liveletters.frontend-app.theme.core :as theme]
            [liveletters.frontend-app.theme.layout :as layout]
            [liveletters.frontend-app.sidebar :as sidebar]
            [liveletters.ui-kit.icons :as icons]))

(defn- nav-icon-button [icon title on-click & [{:keys [accent?]}]]
  [:button {:type "button"
            :title title
            :class (if accent? "ll-button ll-button--primary" "ll-button ll-button--secondary")
            :style {:display "flex"
                    :align-items "center"
                    :justify-content "center"
                    :padding "8px"
                    :min-width "36px"
                    :border-radius "8px"
                    :background (if accent? "var(--accent)" "transparent")
                    :border (if accent? "none" "1px solid rgba(255,255,255,0.08)")
                    :color (if accent? "#fff" "var(--text-secondary)")
                    :cursor "pointer"
                    :transition "background 120ms, color 120ms"}
            :on-click on-click}
   icon])

(defn- top-nav-bar [store]
  [layout/top-nav {:class "ll-top-nav"}
   ;; Левая группа: навигация
   [:div {:style {:display "flex" :align-items "center" :gap "4px"}}
    [nav-icon-button (icons/icon-back) "Назад"
     #(swap! store assoc :route (routes/feed-route))]
    [nav-icon-button (icons/icon-forward) "Вперёд"
     #(swap! store assoc :route (routes/feed-route))]]
   ;; Правая группа: действия
   [:div {:style {:display "flex" :align-items "center" :gap "6px"}}
    [nav-icon-button (icons/icon-pen) "Написать пост"
     #(swap! store assoc :route {:page :editor})
     {:accent? true}]
    [nav-icon-button (icons/icon-plus) "Добавить подписку"
     #(swap! store assoc :route {:page :add-subscription})]
    [nav-icon-button (icons/icon-settings) "Настройки"
     #(swap! store assoc :route (routes/settings-route))]]])

(defn- main-content-area [store state]
  [:div {:style {:padding "24px"}}
   (case (selectors/current-page state)
     :initial-setup (pages/initial-setup-page store state)
     :post (pages/post-page store state)
     :sync (pages/sync-page store state)
     :diagnostics (pages/diagnostics-page store state)
     :settings (pages/settings-page store state)
     (pages/feed-page store state))])

(defn root-view [store]
  (let [state @store
        current-page (selectors/current-page state)
        setup-done? (get-in state [:bootstrap :setup-completed?])]
    (if setup-done?
      ;; Основной layout: sidebar + main content
      [:div {:style {:display "grid"
                     :grid-template-rows "48px 1fr"
                     :grid-template-columns "280px 1fr"
                     :height "100vh"
                     :background "var(--bg-primary)"}}
       ;; Top nav bar (на всю ширину)
       [:div {:style {:grid-row "1" :grid-column "1 / span 2"}}
        [top-nav-bar store]]
       ;; Sidebar
       [sidebar/sidebar-view {:active-page current-page
                              :on-home #(swap! store assoc :route (routes/feed-route))
                              :on-feed #(swap! store assoc :route (routes/feed-route))
                              :on-settings #(swap! store assoc :route (routes/settings-route))}]
       ;; Main content
       [layout/main-content {:class "ll-main"}
        [main-content-area store state]]]
      ;; Initial setup: показываем только форму настроек
      [theme/app-shell {:class "ll-app"}
       [:div {:class "ll-shell"}
        (pages/initial-setup-page store state)]])))
