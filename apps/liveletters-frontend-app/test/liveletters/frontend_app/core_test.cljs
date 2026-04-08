(ns liveletters.frontend-app.core-test
  (:require [cljs.test :refer-macros [deftest is]]
            [liveletters.frontend-app.core :as core]))

(deftest exposes-module-info
  (is (= {:module :liveletters-frontend-app
          :language :cljc}
         (core/module-info))))
