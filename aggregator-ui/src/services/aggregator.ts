import { api } from "./index";
import { aggregator as generatedApi } from './aggregator.generated';

const API_BASE_URL = import.meta.env.VITE_BASE_URL;

const aggregatorApi = api.injectEndpoints({
  endpoints: (build) => ({
    post: build.query<PostApiResponse, PostApiArg>({
      query: () => ({ url: `/post` }),
     async onCacheEntryAdded(_,  { updateCachedData,  cacheEntryRemoved }) {
        const eventSource = new EventSource(`${API_BASE_URL}/post`);

        eventSource.onmessage = (event) => {
          const parsedData: string = JSON.parse(event.data);
          updateCachedData((draft) => {
            draft.push(parsedData);
          });
        };
        
        await cacheEntryRemoved;
        eventSource.close();
     }
    }),
  }),
  overrideExisting: true,
});
export type PostApiResponse = string[];
export type PostApiArg = void;
export const { useHealthCheckQuery } = generatedApi; // Re-export the hooks you want to keep
export const { usePostQuery } = aggregatorApi; 
