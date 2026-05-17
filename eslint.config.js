// ESLint 9 flat config. The legacy .eslintrc.cjs format is incompatible
// with ESLint 9's default behavior — the prompt's `.eslintrc.cjs` shape
// would only work with ESLINT_USE_FLAT_CONFIG=false. Using the modern
// flat-config format here.
import js from '@eslint/js';
import tseslint from 'typescript-eslint';
import react from 'eslint-plugin-react';
import reactHooks from 'eslint-plugin-react-hooks';
import globals from 'globals';

export default tseslint.config(
  {
    ignores: [
      'dist/**',
      'node_modules/**',
      'src-tauri/target/**',
      'src-tauri/gen/**',
      'target/**',
      'playwright-report/**',
      'test-results/**',
      'coverage/**',
      // Node tooling scripts — not part of the typed TS project.
      'bin/**',
      // Generated TypeScript bindings — owned by `cargo xtask regenerate-types`.
      // Per CLAUDE.md §14 schemas-as-source-of-truth.
      'src/types/agent_event.ts',
      'src/types/error.ts',
      'src/types/plan.ts',
      'src/types/task.ts',
      'src/types/hitl.ts',
      'src/types/budget.ts',
      'src/types/capability.ts',
      'src/types/mcp.ts',
    ],
  },
  js.configs.recommended,
  ...tseslint.configs.recommendedTypeChecked,
  {
    languageOptions: {
      globals: { ...globals.browser, ...globals.node },
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
    plugins: {
      react,
      'react-hooks': reactHooks,
    },
    settings: { react: { version: '18.3' } },
    rules: {
      ...react.configs.recommended.rules,
      ...reactHooks.configs.recommended.rules,
      'react/react-in-jsx-scope': 'off',
      '@typescript-eslint/no-unused-vars': [
        'error',
        { argsIgnorePattern: '^_', varsIgnorePattern: '^_' },
      ],
    },
  },
  {
    files: ['tests/**/*.{ts,tsx}'],
    rules: {
      '@typescript-eslint/no-explicit-any': 'off',
      '@typescript-eslint/no-unsafe-assignment': 'off',
      '@typescript-eslint/no-unsafe-member-access': 'off',
      '@typescript-eslint/no-unsafe-call': 'off',
      '@typescript-eslint/no-unsafe-argument': 'off',
      '@typescript-eslint/no-unsafe-return': 'off',
      '@typescript-eslint/unbound-method': 'off',
      // Mock factories in @tauri-apps/api shims have async shape to match
      // the production surface, but their bodies are sync — that's the
      // point of the mock.
      '@typescript-eslint/require-await': 'off',
    },
  },
  {
    files: ['**/*.config.{ts,js}', 'eslint.config.js', 'wdio.conf.ts'],
    languageOptions: {
      parserOptions: { projectService: false, project: null },
    },
    ...tseslint.configs.disableTypeChecked,
  },
);
