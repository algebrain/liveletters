(ns liveletters.ui-kit.core-test
  (:require [cljs.test :refer-macros [deftest is]]
            [liveletters.ui-kit.core :as core]))

(deftest exposes-module-info
  (is (= {:module :liveletters-ui-kit
          :language :cljc}
         (core/module-info))))
