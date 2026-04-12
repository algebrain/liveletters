(ns liveletters.frontend-app.app
  (:require [liveletters.frontend-app.pages :as pages]
            [liveletters.frontend-app.routes :as routes]
            [liveletters.frontend-app.selectors :as selectors]
            [liveletters.frontend-app.theme.core :as theme]
            [liveletters.frontend-app.theme.layout :as layout]
            [liveletters.frontend-app.sidebar :as sidebar]
            [liveletters.frontend-app.content-views :as content]
            [liveletters.frontend-app.modals :as modals]
            [liveletters.ui-kit.icons :as icons]))

(defn- nav-icon-button [icon title on-click & [{:keys [accent?]}]]
  [:button {:type "button"
            :title title
            :class (if accent? "ll-button ll-button--primary" "ll-button ll-button--secondary")
            :style {:display "flex"
                    :align-items "center"
                    :justify-content "center"
                    :padding (if accent? "8px 12px" "8px")
                    :min-width (if accent? "44px" "34px")
                    :height "34px"
                    :border-radius (if accent? "12px" "10px")
                    :background (if accent? "var(--accent)" "rgba(255,255,255,0.02)")
                    :border (if accent? "none" "1px solid var(--border-soft)")
                    :color (if accent? "#fff" "var(--text-secondary)")
                    :cursor "pointer"
                    :box-shadow (if accent? "0 10px 24px rgba(58,115,168,0.22)" "none")
                    :transition "background 120ms, color 120ms, border-color 120ms"}
            :on-click on-click}
   icon])

(defn- top-nav-bar [store]
  [layout/top-nav {:class "ll-top-nav"}
   ;; Левая группа: навигация
   [:div {:style {:display "flex" :align-items "center" :gap "6px"}}
    [nav-icon-button (icons/icon-back) "Назад"
     #(swap! store assoc :route {:page :feed})]
    [nav-icon-button (icons/icon-forward) "Вперёд"
     #(swap! store assoc :route {:page :feed})]]
   ;; Правая группа: действия
   [:div {:style {:display "flex" :align-items "center" :gap "8px"}}
    [nav-icon-button (icons/icon-pen) "Написать пост"
     #(swap! store assoc :route {:page :editor})
     {:accent? true}]
    [nav-icon-button (icons/icon-plus) "Добавить подписку"
     #(swap! store assoc :modal :add-subscription)]
    [nav-icon-button (icons/icon-settings) "Настройки"
     #(swap! store assoc :modal :settings)]]])

(defn- modal-overlay [store state]
  (let [modal (:modal state)]
    (case modal
      :settings
      [modals/settings-modal store state true
       #(swap! store assoc :modal nil)]
      :add-subscription
      [modals/add-subscription-modal
       #(swap! store assoc :modal nil)]
      nil)))

(defn- main-content-area [store state]
  (let [current-page (selectors/current-page state)]
    (case current-page
      ;; Новые стили
      :feed [content/feed-page]
      :home [content/feed-page]
      :post-thread [content/post-thread-page]
      :editor [content/editor-page]
      ;; Старые страницы (сохранены для совместимости)
      :initial-setup (pages/initial-setup-page store state)
      :post (pages/post-page store state)
      :sync (pages/sync-page store state)
      :diagnostics (pages/diagnostics-page store state)
      :settings (pages/settings-page store state)
      ;; По умолчанию — лента
      [content/feed-page])))

(defn root-view [store]
  (let [state @store
        current-page (selectors/current-page state)
        setup-done? (get-in state [:bootstrap :setup-completed?])]
    [:<>
     (if setup-done?
       ;; Основной layout: sidebar + main content
       [:div {:style {:display "grid"
                      :grid-template-rows "46px 1fr"
                      :grid-template-columns "286px 1fr"
                      :height "100vh"
                      :background "var(--bg-app)"}}
        ;; Top nav bar (на всю ширину)
        [:div {:style {:grid-row "1" :grid-column "1 / span 2"}}
         [top-nav-bar store]]
        ;; Sidebar
        [sidebar/sidebar-view {:active-page current-page
                               :on-home #(swap! store assoc :route {:page :home})
                               :on-feed #(swap! store assoc :route {:page :feed})
                               :on-settings #(swap! store assoc :modal :settings)}]
        ;; Main content
        [layout/main-content {:class "ll-main"}
         [main-content-area store state]]]
       ;; Initial setup: показываем только форму настроек
       [theme/app-shell {:class "ll-app"}
        [:div {:class "ll-shell"}
         (pages/initial-setup-page store state)]])
     ;; Модалки поверх всего
     (when setup-done?
       (modal-overlay store state))]))
