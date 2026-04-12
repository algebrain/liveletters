(ns liveletters.frontend-app.theme.core
  "Переэкспорт всех ornament-компонентов темы для удобства require.

   Вместо:
     (ns foo (:require [liveletters.frontend-app.theme.shell :as theme]
                       [liveletters.frontend-app.theme.nav :as theme.nav]))

   Можно:
     (ns foo (:require [liveletters.frontend-app.theme.core :as theme]))"
  (:require [liveletters.frontend-app.theme.shell]
            [liveletters.frontend-app.theme.nav]
            [liveletters.frontend-app.theme.page]
            [liveletters.frontend-app.theme.settings]))

;; Переэкспорт символов для удобства
(def app-shell liveletters.frontend-app.theme.shell/app-shell)
(def nav-shell liveletters.frontend-app.theme.nav/nav-shell)
(def page-copy liveletters.frontend-app.theme.page/page-copy)
(def actions-row liveletters.frontend-app.theme.page/actions-row)
(def settings-layout liveletters.frontend-app.theme.settings/settings-layout)
(def settings-grid liveletters.frontend-app.theme.settings/settings-grid)
(def settings-card liveletters.frontend-app.theme.settings/settings-card)
(def settings-column liveletters.frontend-app.theme.settings/settings-column)
