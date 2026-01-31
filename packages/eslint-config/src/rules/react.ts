/**
 * React-specific ESLint rules for FAANG quality standards
 *
 * This configuration enforces best practices for React development,
 * focusing on performance, accessibility, and maintainability.
 *
 * @module eslint-config/react
 */

import type { Linter } from 'eslint';

export const reactRules: Linter.RulesRecord = {
  // ===========================================================================
  // React Core Rules (CRITICAL)
  // ===========================================================================

  // Hooks enforcement - catch common mistakes
  'react-hooks/rules-of-hooks': 'error',
  'react-hooks/exhaustive-deps': [
    'error',
    {
      additionalHooks: '(use(Dispatch|Selector|FormState|DeepCompareEffect))',
      enableDangerousAutofixThisMayCauseInfiniteLoops: false,
      hooksOrder: 'functions-first',
    },
  ],

  // ===========================================================================
  // Component Design Patterns (CRITICAL)
  // ===========================================================================

  // Component composition over inheritance
  'react/prefer-es6-class': 'off',
  'react/prefer-stateless-function': 'error',
  'react/prefer-read-only-props': 'warn',
  'react/prop-types': 'off', // TypeScript handles this
  'react/require-default-props': 'off', // TypeScript handles this
  'react/require-optimization': 'off', // Let developers decide
  'react/forbid-component-props': 'off', // Too restrictive
  'react/forbid-elements': 'off', // Let developers decide
  'react/forbid-foreign-prop-types': 'warn',
  'react/forbid-prop-types': [
    'warn',
    {
      forbid: ['any'],
      checkContextTypes: true,
      checkChildContextTypes: true,
    },
  ],

  // ===========================================================================
  // JSX Quality (HIGH)
  // ===========================================================================

  // Consistent JSX structure
  'react/jsx-boolean-value': ['error', 'never'],
  'react/jsx-closing-bracket-location': [
    'error',
    {
      nonEmpty: 'after-props',
      selfClosing: 'after-props',
    },
  ],
  'react/jsx-closing-tag-location': 'error',
  'react/jsx-curly-brace-presence': [
    'error',
    {
      props: 'never',
      children: 'never',
      propElementValues: 'ignore',
    },
  ],
  'react/jsx-curly-newline': 'error',
  'react/jsx-curly-spacing': [
    'error',
    'never',
    {
      allowMultiline: true,
    },
  ],
  'react/jsx-equals-spacing': ['error', 'never'],
  'react/jsx-first-prop-new-line': ['error', 'multiline'],
  'react/jsx-fragments': ['error', 'syntax'],
  'react/jsx-handler-names': [
    'error',
    {
      eventHandlerPrefix: 'handle',
      eventHandlerPropPrefix: 'on',
      checkLocalVariables: true,
      checkInlineFunction: false,
    },
  ],
  'react/jsx-indent': ['error', 2],
  'react/jsx-indent-props': ['error', 2],
  'react/jsx-key': [
    'error',
    {
      checkFragmentShorthand: true,
      checkKeyMustBeforeSpread: true,
      warnOnDuplicates: true,
    },
  ],
  'react/jsx-max-depth': ['warn', { max: 10 }],
  'react/jsx-no-bind': 'warn', // Prefer arrow functions in render
  'react/jsx-no-comment-textnodes': 'error',
  'react/jsx-no-duplicate-props': 'error',
  'react/jsx-no-leaked-render': [
    'error',
    {
      validStrategies: ['coerce', 'ternary'],
    },
  ],
  'react/jsx-no-literals': [
    'warn',
    {
      allowedStrings: ['&', ' ', '-', '+', '(', ')'],
      ignoreProps: true,
    },
  ],
  'react/jsx-no-script-url': 'error',
  'react/jsx-no-target-blank': 'error',
  'react/jsx-no-undef': 'error',
  'react/jsx-one-expression-per-line': ['error', { allow: 'single-child' }],
  'react/jsx-pascal-case': [
    'error',
    {
      allowAllCaps: true,
      ignore: [],
    },
  ],
  'react/jsx-props-no-multi-spaces': 'error',
  'react/jsx-props-no-spreading': 'off', // Allow spreading in some cases
  'react/jsx-sort-props': [
    'warn',
    {
      callbacksLast: true,
      shorthandFirst: true,
      shorthandLast: false,
      ignoreCase: true,
      noSortAlphabetically: false,
      reservedFirst: true,
    },
  ],
  'react/jsx-tag-spacing': [
    'error',
    {
      closingSlash: 'never',
      beforeSelfClosing: 'always',
      afterOpening: 'never',
      beforeClosing: 'never',
    },
  ],
  'react/jsx-uses-react': 'off', // Not needed in React 17+
  'react/jsx-uses-vars': 'error',
  'react/jsx-wrap-multilines': [
    'error',
    {
      declaration: 'parens-new-line',
      assignment: 'parens-new-line',
      return: 'parens-new-line',
      arrow: 'parens-new-line',
      condition: 'ignore',
      logical: 'ignore',
      prop: 'ignore',
    },
  ],

  // ===========================================================================
  // Function Components (HIGH)
  // ===========================================================================

  'react/function-component-definition': [
    'error',
    {
      namedComponents: 'function-declaration',
      unnamedComponents: 'arrow-function',
    },
  ],
  'react/hook-use-state': 'warn',
  'react/destructuring-assignment': [
    'warn',
    {
      arrayInDestructuring: 'always',
      objectInDestructuring: 'always',
    },
  ],
  'react/no-children-prop': 'error',
  'react/no-danger': 'warn',
  'react/no-danger-with-children': 'error',
  'react/no-deprecated': 'warn',
  'react/no-did-mount-set-state': 'error',
  'react/no-did-update-set-state': 'error',
  'react/no-direct-mutation-state': 'error',
  'react/no-find-dom-node': 'error',
  'react/no-is-mounted': 'error',
  'react/no-multi-comp': 'off', // Allow multiple components per file
  'react/no-redundant-should-component-update': 'error',
  'react/no-render-return-value': 'error',
  'react/no-set-state': 'off', // Allow setState in class components
  'react/no-string-refs': 'error',
  'react/no-this-in-sfc': 'error',
  'react/no-unescaped-entities': 'error',
  'react/no-unknown-property': 'error',
  'react/no-unsafe': 'warn',
  'react/no-unused-prop-types': 'off', // TypeScript handles this
  'react/no-unused-state': 'warn',
  'react/no-will-update-set-state': 'error',
  'react/prefer-stateless-function': 'error',
  'react/react-in-jsx-scope': 'off', // Not needed in React 17+
  'react/require-render-return': 'error',
  'react/self-closing-comp': 'error',

  // ===========================================================================
  // Performance Optimization (HIGH)
  // ===========================================================================

  'react/jsx-no-useless-fragment': [
    'error',
    {
      allowExpressions: true,
    },
  ],
  'react/no-array-index-key': 'warn',
  'react/prop-types': 'off',
  'react/jsx-no-leaked-render': [
    'error',
    {
      validStrategies: ['coerce', 'ternary', 'logical'],
    },
  ],
  'react-perf/jsx-no-jsx-as-prop': 'warn',
  'react-perf/jsx-no-new-array-as-prop': 'warn',
  'react-perf/jsx-no-new-function-as-prop': 'warn',
  'react-perf/jsx-no-new-object-as-prop': 'warn',

  // ===========================================================================
  // Accessibility (A11Y) (HIGH)
  // ===========================================================================

  'react/jsx-aria-role': 'error',
  'react/jsx-no-duplicate-props': 'error',
  'react/jsx-no-undef': 'error',
  'jsx-a11y/anchor-has-content': 'error',
  'jsx-a11y/anchor-is-valid': 'error',
  'jsx-a11y/aria-activedescendant-has-tabindex': 'error',
  'jsx-a11y/aria-props': 'error',
  'jsx-a11y/aria-proptypes': 'error',
  'jsx-a11y/aria-role': 'error',
  'jsx-a11y/aria-unsupported-elements': 'error',
  'jsx-a11y/click-events-have-key-events': 'warn',
  'jsx-a11y/control-has-associated-label': 'warn',
  'jsx-a11y/heading-has-content': 'error',
  'jsx-a11y/html-has-lang': 'error',
  'jsx-a11y/img-redundant-alt': 'warn',
  'jsx-a11y/interactive-supports-focus': 'error',
  'jsx-a11y/label-has-associated-control': 'error',
  'jsx-a11y/mouse-events-have-key-events': 'error',
  'jsx-a11y/no-access-key': 'error',
  'jsx-a11y/no-autofocus': 'warn',
  'jsx-a11y/no-distracting-elements': 'error',
  'jsx-a11y/no-interactive-element-to-noninteractive-role': 'error',
  'jsx-a11y/no-noninteractive-element-interactions': 'warn',
  'jsx-a11y/no-noninteractive-tabindex': 'error',
  'jsx-a11y/no-redundant-roles': 'error',
  'jsx-a11y/no-static-element-interactions': 'warn',
  'jsx-a11y/role-has-required-aria-props': 'error',
  'jsx-a11y/role-supports-aria-props': 'error',
  'jsx-a11y/scope': 'warn',

  // ===========================================================================
  // Code Quality and Maintainability (MEDIUM)
  // ===========================================================================

  'react/button-has-type': 'error',
  'react/default-props-match-prop-types': 'off',
  'react/no-unstable-nested-components': 'warn',
  'react/hook-use-state': 'warn',
  'react/no-invalid-html-attribute': 'error',
  'react/jsx-props-no-spreading': [
    'warn',
    {
      html: 'enforce',
      custom: 'ignore',
      explicitSpread: 'ignore',
      exceptions: [],
    },
  ],
  'react/jsx-no-leaked-render': [
    'error',
    {
      validStrategies: ['coerce', 'ternary', 'logical'],
    },
  ],
  'react/jsx-no-bind': [
    'warn',
    {
      ignoreRefs: true,
      allowArrowFunctions: true,
      allowBind: false,
    },
  ],

  // ===========================================================================
  // TypeScript + React Integration (MEDIUM)
  // ===========================================================================

  '@typescript-eslint/consistent-type-assertions': 'error',
  '@typescript-eslint/explicit-function-return-type': [
    'off',
    {
      allowExpressions: true,
      allowTypedFunctionExpressions: true,
      allowHigherOrderFunctions: true,
    },
  ],
  '@typescript-eslint/explicit-module-boundary-types': [
    'warn',
    {
      allowArgumentsExplicitlyTypedAsAny: false,
      allowDirectConstAssertionInArrowFunctions: true,
      allowHigherOrderFunctions: true,
    },
  ],

  // ===========================================================================
  // Testing Patterns (LOW)
  // ===========================================================================

  'react/no-children-prop': 'error',
  'react/no-danger-with-children': 'error',
  'react/no-unstable-nested-components': 'warn',
};
