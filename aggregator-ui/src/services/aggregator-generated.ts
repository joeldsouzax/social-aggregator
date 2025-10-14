import { api } from "./index";
const injectedRtkApi = api.injectEndpoints({
  endpoints: (build) => ({
    healthCheck: build.query<HealthCheckApiResponse, HealthCheckApiArg>({
      query: () => ({ url: `/health` }),
    }),
    sse: build.query<SseApiResponse, SseApiArg>({
      query: () => ({ url: `/sse` }),
    }),
  }),
  overrideExisting: false,
});
export { injectedRtkApi as aggregator };
export type HealthCheckApiResponse =
  /** status 200 Application is Healthy */ HealthResponse;
export type HealthCheckApiArg = void;
export type SseApiResponse = unknown;
export type SseApiArg = void;
export type HealthResponse = {
  message: string;
};
export const { useHealthCheckQuery, useSseQuery } = injectedRtkApi;
