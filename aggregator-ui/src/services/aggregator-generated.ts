import { api } from "./index";
const injectedRtkApi = api.injectEndpoints({
  endpoints: (build) => ({
    healthCheck: build.query<HealthCheckApiResponse, HealthCheckApiArg>({
      query: () => ({ url: `/health` }),
    }),
    post: build.query<PostApiResponse, PostApiArg>({
      query: () => ({ url: `/post` }),
    }),
  }),
  overrideExisting: false,
});
export { injectedRtkApi as aggregator };
export type HealthCheckApiResponse =
  /** status 200 Application is Healthy */ HealthResponse;
export type HealthCheckApiArg = void;
export type PostApiResponse = unknown;
export type PostApiArg = void;
export type HealthResponse = {
  message: string;
};
export const { useHealthCheckQuery, usePostQuery } = injectedRtkApi;
