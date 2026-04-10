(ns liveletters.frontend-app.app
  (:require [liveletters.frontend-app.pages :as pages]
            [liveletters.frontend-app.routes :as routes]
            [liveletters.frontend-app.selectors :as selectors]
            [liveletters.frontend-app.theme :as theme]))

(defn root-view [store]
  (let [state @store]
    [theme/app-shell {:class "ll-app"}
     [:div {:class "ll-shell"}
      (when (get-in state [:bootstrap :setup-completed?])
        [theme/nav-shell {:class "ll-nav-shell"}
        [:div {:class "ll-nav-label"} "LiveLetters"]
        [:nav {:class "ll-nav"}
         [:button {:type "button"
                   :class "ll-button ll-button--secondary"
                   :on-click #(swap! store assoc :route (routes/feed-route))}
          "Feed"]
         [:button {:type "button"
                   :class "ll-button ll-button--secondary"
                   :on-click #(swap! store assoc :route (routes/sync-route))}
          "Sync"]
         [:button {:type "button"
                   :class "ll-button ll-button--secondary"
                   :on-click #(swap! store assoc :route (routes/diagnostics-route))}
          "Diagnostics"]
         [:button {:type "button"
                   :class "ll-button ll-button--secondary"
                   :on-click #(swap! store assoc :route (routes/settings-route))}
          "Settings"]]])
      (case (selectors/current-page state)
        :initial-setup (pages/initial-setup-page store state)
        :post (pages/post-page store state)
        :sync (pages/sync-page store state)
        :diagnostics (pages/diagnostics-page store state)
        :settings (pages/settings-page store state)
        (pages/feed-page store state))]]))
