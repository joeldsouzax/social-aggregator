import type { ConfigFile } from '@rtk-query/codegen-openapi'

const config: ConfigFile = {
  schemaFile: 'http://localhost:3000/api-docs/openapi.json',
  apiFile: './src/services/index.ts',
  apiImport: 'api',
  outputFile: './src/services/aggregator-generated.ts',
  exportName: 'aggregator',
  hooks: true,
}

export default config
