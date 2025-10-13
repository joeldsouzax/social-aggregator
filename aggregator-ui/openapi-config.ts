import type { ConfigFile } from '@rtk-query/codegen-openapi'

const config: ConfigFile = {
  schemaFile: 'http://localhost:3000/api-docs/openapi.json',
  apiFile: './src/store/aggregatorApi.ts',
  apiImport: 'aggregatorApi',
  outputFile: './src/store/aggregator.ts',
  exportName: 'aggregator',
  hooks: true,
}

export default config
