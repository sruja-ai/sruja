// @sruja/eslint-config - Base ESLint configuration for TypeScript projects
// ESLint v9 flat config format with FAANG quality standards
//
// This configuration enforces:
// - Strict type safety with comprehensive TypeScript rules
// - Code quality patterns used at FAANG companies
// - Consistent code style and organization
// - Performance and maintainability best practices

import js from "@eslint/js";
import tseslint from "typescript-eslint";

// ============================================================================
// TypeScript-specific rules for FAANG quality standards
// ============================================================================

const typescriptStrictRules = {
  // ===========================================================================
  // Type Safety Rules (CRITICAL)
  // ===========================================================================

  // Ensure variables are properly typed
  "@typescript-eslint/no-explicit-any": "error",
  "@typescript-eslint/no-unsafe-assignment": "error",
  "@typescript-eslint/no-unsafe-call": "error",
  "@typescript-eslint/no-unsafe-member-access": "error",
  "@typescript-eslint/no-unsafe-return": "error",
  "@typescript-eslint/no-unsafe-argument": "error",
  "@typescript-eslint/no-unsafe-enum-comparison": "error",

  // Strict type checking
  "@typescript-eslint/strict-boolean-expressions": [
    "error",
    {
      allowString: false,
      allowNumber: false,
      allowNullableObject: false,
      allowNullableBoolean: false,
      allowNullableString: false,
      allowNullableNumber: false,
    },
  ],
  "@typescript-eslint/no-unnecessary-type-assertion": "error",
  "@typescript-eslint/no-unnecessary-type-arguments": "error",
  "@typescript-eslint/no-unnecessary-type-constraint": "error",
  "@typescript-eslint/no-unnecessary-type-parameters": "error",
  "@typescript-eslint/no-unnecessary-qualifier": "error",
  "@typescript-eslint/no-unnecessary-condition": "error",
  "@typescript-eslint/no-unnecessary-boolean-literal-compare": "error",
  "@typescript-eslint/no-confusing-non-null-assertion": "error",
  "@typescript-eslint/no-floating-promises": "error",
  "@typescript-eslint/no-misused-new": "error",
  "@typescript-eslint/no-misused-promises": [
    "error",
    {
      checksConditionals: true,
      checksVoidReturn: false,
      checksSpreads: true,
    },
  ],
  "@typescript-eslint/prefer-nullish-coalescing": [
    "error",
    {
      ignoreMixedLogicalExpressions: true,
      ignoreConditionalTests: false,
      ignorePrimitives: false,
    },
  ],
  "@typescript-eslint/prefer-optional-chain": "error",
  "@typescript-eslint/prefer-reduce-type-parameter": "error",
  "@typescript-eslint/prefer-includes": "error",
  "@typescript-eslint/prefer-string-starts-ends-with": "error",
  "@typescript-eslint/prefer-as-const": "error",
  "@typescript-eslint/prefer-ts-expect-error": "error",

  // ===========================================================================
  // Code Quality Rules (HIGH)
  // ===========================================================================

  // Ban harmful patterns
  "@typescript-eslint/ban-ts-comment": [
    "error",
    {
      "ts-expect-error": "allow-with-description",
      "ts-ignore": false,
      "ts-nocheck": false,
      "ts-check": false,
      minimumDescriptionLength: 10,
    },
  ],
  "@typescript-eslint/ban-tslint-comment": "error",
  "@typescript-eslint/no-duplicate-enum-values": "error",
  "@typescript-eslint/no-duplicate-type-constituents": "error",
  "@typescript-eslint/no-empty-interface": [
    "error",
    {
      allowSingleExtends: true,
    },
  ],
  "@typescript-eslint/no-invalid-void-type": "error",
  "@typescript-eslint/no-meaningless-void-operator": "error",
  "@typescript-eslint/no-magic-numbers": [
    "warn",
    {
      ignore: [-1, 0, 1, 2, 100, 1000],
      ignoreArrayIndexes: true,
      ignoreDefaultValues: true,
      ignoreEnums: true,
      ignoreNumericLiteralTypes: true,
      ignoreReadonlyClassProperties: true,
      ignorePropertyTypeChanges: true,
    },
  ],
  "@typescript-eslint/no-non-null-assertion": "warn", // Prefer optional chaining
  "@typescript-eslint/no-redundant-type-constituents": "error",
  "@typescript-eslint/no-require-imports": "error",
  "@typescript-eslint/no-var-requires": "error",

  // ===========================================================================
  // Best Practices (HIGH)
  // ===========================================================================

  // Consistent naming and style
  "@typescript-eslint/naming-convention": [
    "error",
    {
      selector: "interface",
      format: ["PascalCase"],
      custom: {
        regex: "^I[A-Z]",
        match: false,
      },
    },
    {
      selector: "typeAlias",
      format: ["PascalCase"],
    },
    {
      selector: "enum",
      format: ["PascalCase"],
    },
    {
      selector: "enumMember",
      format: ["PascalCase"],
    },
    {
      selector: "class",
      format: ["PascalCase"],
    },
    {
      selector: "method",
      format: ["camelCase"],
    },
    {
      selector: "function",
      format: ["camelCase"],
    },
    {
      selector: "variable",
      format: ["camelCase", "UPPER_CASE"],
    },
    {
      selector: "parameter",
      format: ["camelCase"],
    },
    {
      selector: "property",
      format: ["camelCase"],
    },
  ],

  // Consistent types
  "@typescript-eslint/array-type": [
    "error",
    {
      default: "array-simple",
      readonly: "array-simple",
    },
  ],
  "@typescript-eslint/consistent-generic-constructors": "error",
  "@typescript-eslint/consistent-indexed-object-style": "error",
  "@typescript-eslint/consistent-type-definitions": ["error", "interface"],
  "@typescript-eslint/consistent-type-exports": "error",
  "@typescript-eslint/consistent-type-imports": [
    "error",
    {
      prefer: "type-imports",
      fixMixedExportsWithInlineTypeSpecifier: true,
      disallowTypeAnnotations: true,
    },
  ],
  "@typescript-eslint/consistent-type-assertions": [
    "error",
    {
      assertionStyle: "as",
      objectLiteralTypeAssertions: "allow-as-parameter",
    },
  ],
  "@typescript-eslint/member-ordering": [
    "warn",
    {
      default: [
        "static-field",
        "public-static-field",
        "protected-static-field",
        "private-static-field",
        "static-method",
        "public-static-method",
        "protected-static-method",
        "private-static-method",
        "instance-field",
        "public-instance-field",
        "protected-instance-field",
        "private-instance-field",
        "constructor",
        "public-instance-method",
        "protected-instance-method",
        "private-instance-method",
      ],
    },
  ],

  // ===========================================================================
  // Performance and Maintainability (MEDIUM)
  // ===========================================================================

  // Prevent performance issues
  "@typescript-eslint/no-for-in-array": "error",
  "@typescript-eslint/no-mixed-enums": "error",
  "@typescript-eslint/no-namespace": "error",
  "@typescript-eslint/no-this-alias": "warn",
  "@typescript-eslint/no-throw-literal": "error",
  "@typescript-eslint/return-await": ["error", "in-try-catch"],
  "@typescript-eslint/await-thenable": "error",
  "@typescript-eslint/promise-function-async": "warn",

  // ===========================================================================
  // Functional Programming Patterns (MEDIUM)
  // ===========================================================================

  "@typescript-eslint/prefer-function-type": "error",
  "@typescript-eslint/prefer-literal-enum-member": "error",
  "@typescript-eslint/prefer-enum-initializers": "error",
  "@typescript-eslint/prefer-namespace-keyword": "error",
  "@typescript-eslint/prefer-regex-literals": "warn",
  "@typescript-eslint/unbound-method": [
    "warn",
    {
      ignoreStatic: true,
    },
  ],

  // ===========================================================================
  // Code Structure and Organization (LOW)
  // ===========================================================================

  "@typescript-eslint/no-extraneous-class": "warn",
  "@typescript-eslint/no-import-type-side-effects": "error",
  "@typescript-eslint/no-unused-vars": [
    "error",
    {
      argsIgnorePattern: "^_",
      varsIgnorePattern: "^_",
      caughtErrorsIgnorePattern: "^_",
      ignoreRestSiblings: true,
    },
  ],
  "@typescript-eslint/no-use-before-define": [
    "error",
    {
      functions: false,
      classes: false,
      variables: true,
      enums: false,
      typedefs: false,
    },
  ],

  // ===========================================================================
  // Type Checking with Additional Context (LOW)
  // ===========================================================================

  "@typescript-eslint/no-dynamic-delete": "warn",
  "@typescript-eslint/no-extra-non-null-assertion": "error",
  "@typescript-eslint/no-implied-eval": "error",
  "@typescript-eslint/no-loop-func": "warn",
  "@typescript-eslint/no-shadow": [
    "warn",
    {
      hoist: "all",
      allow: ["catch", "declare"],
      ignoreTypeValueShadow: true,
      ignoreFunctionTypeParameterNameValueShadow: true,
    },
  ],
  "@typescript-eslint/no-unnecessary-type-parameters": "warn",
  "@typescript-eslint/parameter-properties": "off", // Allow parameter properties
  "@typescript-eslint/class-methods-use-this": "warn",

  // ===========================================================================
  // Modern TypeScript Features (LOW)
  // ===========================================================================

  "@typescript-eslint/use-unknown-in-catch-callback-variable": "error",
  "@typescript-eslint/no-base-to-string": "warn",
  "@typescript-eslint/no-confusing-void-expression": "warn",
  "@typescript-eslint/prefer-destructuring": [
    "warn",
    {
      array: true,
      object: true,
    },
    {
      enforceForRenamedProperties: false,
    },
  ],
  "@typescript-eslint/switch-exhaustiveness-check": "error",
};

// ============================================================================
// General code quality rules
// ============================================================================

const generalQualityRules = {
  // ===========================================================================
  // Code Quality (CRITICAL)
  // ===========================================================================

  "no-console": [
    "error",
    {
      allow: ["warn", "error"],
    },
  ],
  "no-debugger": "error",
  "no-alert": "error",
  "no-eval": "error",
  "no-implied-eval": "error",
  "no-throw-literal": "error",
  "no-unreachable": "error",
  "no-unsafe-finally": "error",
  "no-unsafe-negation": "error",
  "no-unsafe-optional-chaining": "error",

  // ===========================================================================
  // Best Practices (HIGH)
  // ===========================================================================

  curly: ["error", "all"],
  "default-case": "error",
  "default-case-last": "error",
  "default-param-last": "error",
  eqeqeq: ["error", "always", { null: "ignore" }],
  "no-caller": "error",
  "no-case-declarations": "error",
  "no-constructor-return": "error",
  "no-else-return": "error",
  "no-empty-function": "warn",
  "no-empty-pattern": "error",
  "no-eq-null": "error",
  "no-eval": "error",
  "no-extend-native": "error",
  "no-extra-label": "error",
  "no-fallthrough": "error",
  "no-floating-decimal": "error",
  "no-implicit-globals": "error",
  "no-iterator": "error",
  "no-labels": "error",
  "no-lone-blocks": "error",
  "no-multi-spaces": "error",
  "no-multi-str": "error",
  "no-new": "error",
  "no-new-func": "error",
  "no-new-wrappers": "error",
  "no-nonoctal-decimal-escape": "error",
  "no-octal-escape": "error",
  "no-param-reassign": "warn",
  "no-proto": "error",
  "no-redeclare": "error",
  "no-return-assign": "error",
  "no-return-await": "error",
  "no-self-compare": "error",
  "no-sequences": "error",
  "no-throw-literal": "error",
  "no-unmodified-loop-condition": "error",
  "no-unused-expressions": ["error", { allowShortCircuit: true, allowTernary: true }],
  "no-useless-call": "error",
  "no-useless-catch": "error",
  "no-useless-concat": "error",
  "no-useless-escape": "error",
  "no-useless-return": "error",
  "no-var": "error",
  "prefer-const": "error",
  "prefer-promise-reject-errors": "error",
  "prefer-regex-literals": "error",
  radix: "error",
  "require-await": "error",
  yoda: "error",

  // ===========================================================================
  // Code Style (MEDIUM)
  // ===========================================================================

  "array-bracket-newline": ["warn", "consistent"],
  "array-bracket-spacing": ["warn", "never"],
  "array-element-newline": "off",
  "block-spacing": ["warn", "always"],
  "brace-style": ["warn", "1tbs", { allowSingleLine: true }],
  camelcase: ["warn", { properties: "never", ignoreDestructuring: true }],
  "capitalized-comments": "off",
  "comma-dangle": [
    "warn",
    {
      arrays: "always-multiline",
      objects: "always-multiline",
      imports: "always-multiline",
      exports: "always-multiline",
      functions: "never",
    },
  ],
  "comma-spacing": ["warn", { before: false, after: true }],
  "comma-style": ["warn", "last"],
  "computed-property-spacing": ["warn", "never"],
  "eol-last": ["warn", "always"],
  "func-call-spacing": ["warn", "never"],
  "func-name-matching": ["warn", "always"],
  "func-names": "off",
  "func-style": ["warn", "declaration", { allowArrowFunctions: true }],
  indent: [
    "warn",
    2,
    {
      SwitchCase: 1,
      VariableDeclarator: "first",
      MemberExpression: 1,
    },
  ],
  "key-spacing": ["warn", { beforeColon: false, afterColon: true }],
  "keyword-spacing": ["warn", { before: true, after: true }],
  "linebreak-style": ["warn", "unix"],
  "lines-between-class-members": ["warn", "always", { exceptAfterSingleLine: true }],
  "max-depth": ["warn", 5],
  "max-len": [
    "warn",
    {
      code: 120,
      tabWidth: 2,
      ignoreUrls: true,
      ignoreComments: true,
      ignoreStrings: true,
      ignoreTemplateLiterals: true,
    },
  ],
  "max-lines": ["warn", { max: 1000, skipBlankLines: true, skipComments: true }],
  "max-lines-per-function": ["warn", { max: 50, skipBlankLines: true, skipComments: true }],
  "max-nested-callbacks": ["warn", 4],
  "max-params": ["warn", { max: 5 }],
  "max-statements": ["warn", { max: 30 }],
  "max-statements-per-line": ["warn", { max: 1 }],
  "multiline-comment-style": ["warn", "separate-multiple-lines"],
  "new-cap": ["warn", { newIsCap: true, capIsNew: false }],
  "new-parens": "warn",
  "no-array-constructor": "warn",
  "no-lonely-if": "warn",
  "no-mixed-operators": [
    "warn",
    {
      groups: [
        ["==", "!=", "===", "!==", ">", ">=", "<", "<="],
        ["&&", "||"],
      ],
      allowSamePrecedence: true,
    },
  ],
  "no-mixed-spaces-and-tabs": "error",
  "no-multi-assign": "warn",
  "no-multiple-empty-lines": ["warn", { max: 2, maxEOF: 1, maxBOF: 0 }],
  "no-negated-condition": "warn",
  "no-nested-ternary": "warn",
  "no-new-object": "warn",
  "no-tabs": "error",
  "no-trailing-spaces": "warn",
  "no-unneeded-ternary": "warn",
  "no-whitespace-before-property": "warn",
  "nonblock-statement-body-position": ["warn", "beside"],
  "object-curly-spacing": ["warn", "always"],
  "object-curly-newline": [
    "warn",
    {
      ObjectExpression: { consistent: true, multiline: true },
      ObjectPattern: { consistent: true, multiline: true },
    },
  ],
  "object-property-newline": ["warn", { allowAllPropertiesOnSameLine: false }],
  "one-var": ["warn", "never"],
  "operator-assignment": ["warn", "always"],
  "operator-linebreak": ["warn", "before"],
  "padded-blocks": ["warn", "never"],
  "quote-props": ["warn", "as-needed"],
  quotes: ["warn", "single", { avoidEscape: true, allowTemplateLiterals: true }],
  semi: ["warn", "always"],
  "semi-spacing": ["warn", { before: false, after: true }],
  "semi-style": ["warn", "last"],
  "space-before-blocks": ["warn", "always"],
  "space-before-function-paren": [
    "warn",
    {
      anonymous: "always",
      named: "never",
      asyncArrow: "always",
    },
  ],
  "space-in-parens": ["warn", "never"],
  "space-infix-ops": "warn",
  "space-unary-ops": ["warn", { words: true, nonwords: false }],
  "spaced-comment": [
    "warn",
    "always",
    {
      line: {
        markers: ["/"],
        exceptions: ["-", "+"],
        space: { markers: ["/", "*"], exceptions: ["-", "*"] },
      },
      block: { balanced: true, markers: ["!"], exceptions: ["*"] },
    },
  ],
  "switch-colon-spacing": ["warn", { after: true, before: false }],
  "template-curly-spacing": ["warn", "never"],
  "wrap-iife": ["warn", "any"],
  "wrap-regex": "warn",
  "yield-star-spacing": ["warn", "both"],
};

// ============================================================================
// Main configuration export
// ============================================================================

export default tseslint.config(
  // ===========================================================================
  // Base recommended rules
  // ===========================================================================
  js.configs.recommended,

  // ===========================================================================
  // TypeScript recommended rules
  // ===========================================================================
  ...tseslint.configs.recommended,

  // ===========================================================================
  // Strict TypeScript rules for library code (default)
  // ===========================================================================
  {
    files: ["**/*.ts", "**/*.tsx", "**/*.mts"],
    rules: {
      ...typescriptStrictRules,
      ...generalQualityRules,
      // Disable some rules that are covered by TypeScript
      "no-unused-vars": "off", // Use TypeScript's no-unused-vars
      "@typescript-eslint/explicit-module-boundary-types": "off", // Infer return types
    },
  },

  // ===========================================================================
  // Allow console and relaxed rules for scripts (CLI tools)
  // ===========================================================================
  {
    files: ["**/scripts/**/*.ts", "**/scripts/**/*.mts"],
    rules: {
      "no-console": "off", // Scripts are CLI tools, console is appropriate
      "@typescript-eslint/no-magic-numbers": "off", // Allow magic numbers in scripts
      "@typescript-eslint/explicit-module-boundary-types": "off", // Scripts can infer types
    },
  },

  // ===========================================================================
  // Test file configuration
  // ===========================================================================
  {
    files: [
      "**/*.test.ts",
      "**/*.test.tsx",
      "**/__tests__/**/*.ts",
      "**/__tests__/**/*.tsx",
      "**/*.spec.ts",
      "**/*.spec.tsx",
    ],
    rules: {
      "@typescript-eslint/no-magic-numbers": "off", // Allow magic numbers in tests
      "@typescript-eslint/no-explicit-any": "warn", // Allow any in tests with warning
      "@typescript-eslint/no-non-null-assertion": "off", // Allow non-null assertions in tests
      "no-console": "off", // Allow console.log for debugging tests
    },
  },

  // ===========================================================================
  // Ignore patterns
  // ===========================================================================
  {
    ignores: [
      "dist/",
      "build/",
      "node_modules/",
      "**/*.js",
      "**/*.js.map",
      "**/*.cjs",
      "*.d.ts",
      "coverage/",
      ".turbo/",
      "pkg/export/html/*.html",
      "storybook-static/",
      ".astro/",
      "out/",
      ".next/",
      "*.config.js",
      "*.config.ts",
      "turbo.json",
      ".eslintrc.*",
    ],
  }
);
