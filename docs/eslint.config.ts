import panzerjack from '@panzerjack/eslint-config'

export default panzerjack({
  typescript: true,
  vue: true,
  pnpm: true,
  formatters: true,
  markdown: false,
  rules: {
    'no-console': 'off',
    'no-irregular-whitespace': 'off',
    'style/no-tabs': 'off',
    'style/no-mixed-spaces-and-tabs': 'off',
    'eslint-comments/no-unlimited-disable': 'off',
  },
})
