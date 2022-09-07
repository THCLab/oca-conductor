module.exports = {
  testTimeout: 15000,
  preset: 'ts-jest',
  globals: {
    'ts-jest': {
      tsconfig: "config/transformer/tsconfig.test.json"
    },
  },
  setupFilesAfterEnv: [],
  testEnvironment: 'node',
  roots: [
    "../../pkg/transformer/src",
    "../../pkg/transformer/test",
  ],
  moduleDirectories: [
    "node_modules",
    "src",
  ],
  moduleNameMapper: {
    '^@test(.*)$': "../../pkg/transformer/test/$1",
  },
  clearMocks: true,
};
