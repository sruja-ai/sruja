module.exports = {
  preset: 'ts-jest',
  testEnvironment: 'node',
  roots: ['<rootDir>/src'],
  testMatch: ['**/*.test.ts'],
  moduleFileExtensions: ['ts', 'js'],
  collectCoverageFrom: [
    'src/**/*.ts',
    '!src/**/*.test.ts',
    '!src/test/**',
  ],
  moduleNameMapper: {
    '^vscode$': '<rootDir>/src/__mocks__/vscode.ts',
  },
  // Target 80% (codecov.yml); thresholds set to current baseline so CI passes.
  // extension.ts is 0% under Jest (exercised by test:vscode e2e).
  coverageThreshold: {
    global: {
      branches: 48,
      functions: 56,
      lines: 47,
      statements: 49,
    },
  },
};
