module.exports = {
  testTimeout: 15000,
  preset: 'ts-jest',
  globals: {
    'ts-jest': {
      tsconfig: "config/validator/tsconfig.test.json"
    },
  },
  setupFilesAfterEnv: [],
  testEnvironment: 'node',
  roots: [
    "../../pkg/validator/src",
    "../../pkg/validator/test",
  ],
  moduleDirectories: [
    "node_modules",
    "src",
  ],
  moduleNameMapper: {
    '^@test(.*)$': "../../pkg/validator/test/$1",
  },
  clearMocks: true,
};
