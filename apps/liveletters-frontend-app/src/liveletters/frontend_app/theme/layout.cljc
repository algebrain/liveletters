(ns liveletters.frontend-app.theme.layout
  "Компоненты трёхпанельного layout: top-nav, sidebar, main-content."
  (:require [lambdaisland.ornament :as o]))

(o/defstyled top-nav :nav
  {:grid-row "1"
   :grid-column "1 / span 2"
   :display "flex"
   :align-items "center"
   :justify-content "space-between"
   :padding "0 16px"
   :height "48px"
   :background "var(--bg-secondary)"
   :border-bottom "1px solid rgba(255, 255, 255, 0.06)"})

(o/defstyled sidebar :aside
  {:grid-row "2"
   :grid-column "1"
   :width "280px"
   :background "var(--bg-secondary)"
   :border-right "1px solid rgba(255, 255, 255, 0.06)"
   :overflow-y "auto"
   :display "flex"
   :flex-direction "column"})

(o/defstyled main-content :main
  {:grid-row "2"
   :grid-column "2"
   :overflow-y "auto"
   :background "var(--bg-primary)"
   :padding "0"})
