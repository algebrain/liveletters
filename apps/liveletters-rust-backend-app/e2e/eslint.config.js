import tsParser from "@typescript-eslint/parser";

export default [
  {
    files: ["**/*.ts"],
    languageOptions: {
      ecmaVersion: 2022,
      sourceType: "module",
      parser: tsParser,
      globals: {
        test: "readonly",
        expect: "readonly",
        page: "readonly",
        browser: "readonly",
        context: "readonly",
      },
    },
    rules: {
      "no-unused-vars": "warn",
    },
  },
];
