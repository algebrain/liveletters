(ns liveletters.frontend-api.core-test
  (:require [cljs.test :refer-macros [deftest is]]
            [liveletters.frontend-api.core :as core]))

(deftest exposes-module-info
  (is (= {:module :liveletters-frontend-api
          :language :cljc}
         (core/module-info))))
