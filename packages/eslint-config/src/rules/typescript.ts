/**
 * TypeScript-specific ESLint rules for FAANG quality standards
 *
 * This configuration enforces best practices for TypeScript development,
 * focusing on type safety, code quality, and maintainability.
 *
 * @module eslint-config/typescript
 */

import type { Linter } from 'eslint';

export const typescriptRules: Linter.RulesRecord = {
  // ===========================================================================
  // Type Safety Rules (CRITICAL)
  // ===========================================================================

  // Ensure variables are properly typed
  '@typescript-eslint/no-explicit-any': 'error',
  '@typescript-eslint/no-unsafe-assignment': 'error',
  '@typescript-eslint/no-unsafe-call': 'error',
  '@typescript-eslint/no-unsafe-member-access': 'error',
  '@typescript-eslint/no-unsafe-return': 'error',
  '@typescript-eslint/no-unsafe-argument': 'error',
  '@typescript-eslint/no-unsafe-declaration-merging': 'warn',
  '@typescript-eslint/no-unsafe-enum-comparison': 'error',

  // Strict type checking
  '@typescript-eslint/strict-boolean-expressions': [
    'error',
    {
      allowString: false,
      allowNumber: false,
      allowNullableObject: false,
      allowNullableBoolean: false,
      allowNullableString: false,
      allowNullableNumber: false,
    },
  ],
  '@typescript-eslint/no-unnecessary-type-assertion': 'error',
  '@typescript-eslint/no-unnecessary-type-arguments': 'error',
  '@typescript-eslint/no-unnecessary-type-constraint': 'error',
  '@typescript-eslint/no-unnecessary-type-parameters': 'error',
  '@typescript-eslint/no-unnecessary-qualifier': 'error',
  '@typescript-eslint/no-unnecessary-condition': 'error',
  '@typescript-eslint/no-unnecessary-boolean-literal-compare': 'error',
  '@typescript-eslint/no-confusing-non-null-assertion': 'error',
  '@typescript-eslint/no-floating-promises': 'error',
  '@typescript-eslint/no-misused-new': 'error',
  '@typescript-eslint/no-misused-promises': [
    'error',
    {
      checksConditionals: true,
      checksVoidReturn: false,
      checksSpreads: true,
    },
  ],
  '@typescript-eslint/prefer-nullish-coalescing': [
    'error',
    {
      ignoreMixedLogicalExpressions: true,
      ignoreConditionalTests: false,
      ignorePrimitives: false,
    },
  ],
  '@typescript-eslint/prefer-optional-chain': 'error',
  '@typescript-eslint/prefer-reduce-type-parameter': 'error',
  '@typescript-eslint/prefer-includes': 'error',
  '@typescript-eslint/prefer-string-starts-ends-with': 'error',
  '@typescript-eslint/prefer-readonly': 'error',
  '@typescript-eslint/prefer-readonly-parameter-types': 'off', // Too strict for most projects
  '@typescript-eslint/prefer-as-const': 'error',
  '@typescript-eslint/prefer-ts-expect-error': 'error',

  // ===========================================================================
  // Code Quality Rules (HIGH)
  // ===========================================================================

  // Ban harmful patterns
  '@typescript-eslint/ban-ts-comment': [
    'error',
    {
      'ts-expect-error': 'allow-with-description',
      'ts-ignore': false,
      'ts-nocheck': false,
      'ts-check': false,
      minimumDescriptionLength: 10,
    },
  ],
  '@typescript-eslint/ban-tslint-comment': 'error',
  '@typescript-eslint/no-duplicate-enum-values': 'error',
  '@typescript-eslint/no-duplicate-type-constituents': 'error',
  '@typescript-eslint/no-empty-interface': [
    'error',
    {
      allowSingleExtends: true,
    },
  ],
  '@typescript-eslint/no-inferrable-types': 'off', // Allow explicit types for clarity
  '@typescript-eslint/no-invalid-void-type': 'error',
  '@typescript-eslint/no-meaningless-void-operator': 'error',
  '@typescript-eslint/no-magic-numbers': [
    'warn',
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
  '@typescript-eslint/no-non-null-assertion': 'warn', // Prefer optional chaining
  '@typescript-eslint/no-redundant-type-constituents': 'error',
  '@typescript-eslint/no-require-imports': 'error',
  '@typescript-eslint/no-unnecessary-type-assertion': 'error',
  '@typescript-eslint/no-var-requires': 'error',

  // ===========================================================================
  // Best Practices (HIGH)
  // ===========================================================================

  // Consistent naming and style
  '@typescript-eslint/naming-convention': [
    'error',
    {
      selector: 'interface',
      format: ['PascalCase'],
      custom: {
        regex: '^I[A-Z]',
        match: false,
      },
    },
    {
      selector: 'typeAlias',
      format: ['PascalCase'],
    },
    {
      selector: 'enum',
      format: ['PascalCase'],
    },
    {
      selector: 'enumMember',
      format: ['PascalCase'],
    },
    {
      selector: 'class',
      format: ['PascalCase'],
    },
    {
      selector: 'method',
      format: ['camelCase'],
    },
    {
      selector: 'function',
      format: ['camelCase'],
    },
    {
      selector: 'variable',
      format: ['camelCase', 'UPPER_CASE'],
    },
    {
      selector: 'parameter',
      format: ['camelCase'],
    },
    {
      selector: 'property',
      format: ['camelCase'],
    },
  ],

  // Consistent types
  '@typescript-eslint/array-type': [
    'error',
    {
      default: 'array-simple',
      readonly: 'array-simple',
    },
  ],
  '@typescript-eslint/consistent-generic-constructors': 'error',
  '@typescript-eslint/consistent-indexed-object-style': 'error',
  '@typescript-eslint/consistent-type-definitions': [
    'error',
    'interface',
  ],
  '@typescript-eslint/consistent-type-exports': 'error',
  '@typescript-eslint/consistent-type-imports': [
    'error',
    {
      prefer: 'type-imports',
      fixMixedExportsWithInlineTypeSpecifier: true,
      disallowTypeAnnotations: true,
    },
  ],
  '@typescript-eslint/consistent-type-assertions': [
    'error',
    {
      assertionStyle: 'as',
      objectLiteralTypeAssertions: 'allow-as-parameter',
    },
  ],
  '@typescript-eslint/member-ordering': [
    'warn',
    {
      default: [
        'static-field',
        'public-static-field',
        'protected-static-field',
        'private-static-field',
        'static-method',
        'public-static-method',
        'protected-static-method',
        'private-static-method',
        'instance-field',
        'public-instance-field',
        'protected-instance-field',
        'private-instance-field',
        'constructor',
        'public-instance-method',
        'protected-instance-method',
        'private-instance-method',
      ],
    },
  ],

  // ===========================================================================
  // Performance and Maintainability (MEDIUM)
  // ===========================================================================

  // Prevent performance issues
  '@typescript-eslint/no-for-in-array': 'error',
  '@typescript-eslint/no-mixed-enums': 'error',
  '@typescript-eslint/no-namespace': 'error',
  '@typescript-eslint/no-this-alias': 'warn',
  '@typescript-eslint/no-throw-literal': 'error',
  '@typescript-eslint/return-await': [
    'error',
    'in-try-catch',
  ],
  '@typescript-eslint/await-thenable': 'error',
  '@typescript-eslint/promise-function-async': 'warn',

  // ===========================================================================
  // Functional Programming Patterns (MEDIUM)
  // ===========================================================================

  '@typescript-eslint/prefer-function-type': 'error',
  '@typescript-eslint/prefer-literal-enum-member': 'error',
  '@typescript-eslint/prefer-enum-initializers': 'error',
  '@typescript-eslint/prefer-namespace-keyword': 'error',
  '@typescript-eslint/prefer-regex-literals': 'warn',
  '@typescript-eslint/unbound-method': [
    'warn',
    {
      ignoreStatic: true,
    },
  ],

  // ===========================================================================
  // Code Structure and Organization (LOW)
  // ===========================================================================

  '@typescript-eslint/no-extraneous-class': 'warn',
  '@typescript-eslint/no-import-type-side-effects': 'error',
  '@typescript-eslint/no-unused-vars': [
    'error',
    {
      argsIgnorePattern: '^_',
      varsIgnorePattern: '^_',
      caughtErrorsIgnorePattern: '^_',
      ignoreRestSiblings: true,
    },
  ],
  '@typescript-eslint/no-use-before-define': [
    'error',
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

  '@typescript-eslint/no-dynamic-delete': 'warn',
  '@typescript-eslint/no-extra-non-null-assertion': 'error',
  '@typescript-eslint/no-implied-eval': 'error',
  '@typescript-eslint/no-loop-func': 'warn',
  '@typescript-eslint/no-shadow': [
    'warn',
    {
      hoist: 'all',
      allow: ['catch', 'declare'],
      ignoreTypeValueShadow: true,
      ignoreFunctionTypeParameterNameValueShadow: true,
    },
  ],
  '@typescript-eslint/no-unnecessary-type-parameters': 'warn',
  '@typescript-eslint/parameter-properties': 'off', // Allow parameter properties
  '@typescript-eslint/class-methods-use-this': 'warn',

  // ===========================================================================
  // Modern TypeScript Features (LOW)
  // ===========================================================================

  '@typescript-eslint/use-unknown-in-catch-callback-variable': 'error',
  '@typescript-eslint/no-import-type-side-effects': 'error',
  '@typescript-eslint/no-base-to-string': 'warn',
  '@typescript-eslint/no-confusing-void-expression': 'warn',
  '@typescript-eslint/prefer-destructuring': [
    'warn',
    {
      array: true,
      object: true,
    },
    {
      enforceForRenamedProperties: false,
    },
  ],
  '@typescript-eslint/prefer-nullish-coalescing': 'error',
  '@typescript-eslint/prefer-optional-chain': 'error',
  '@typescript-eslint/switch-exhaustiveness-check': 'error',
};
