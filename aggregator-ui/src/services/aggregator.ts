import { api } from "./index";
import { aggregator as generatedApi } from './aggregator-generated';

const API_BASE_URL = import.meta.env.VITE_BASE_URL;

const aggregatorApi = api.injectEndpoints({
  endpoints: (build) => ({
    post: build.query<string[], string>({
     queryFn: () => ({ data: [] }),
     keepUnusedDataFor: 0,
      async onCacheEntryAdded(
        _,
        { updateCachedData, cacheEntryRemoved }
      ) {
        const eventSource = new EventSource(`${API_BASE_URL}/post`);

        eventSource.onmessage = (event) => {
          try {
            const parsedEvent: string = event.data;
            updateCachedData((draft) => {
                draft.push(parsedEvent);
            });
          } catch (e) {
            console.error('Failed to parse SSE event:', e);
          }
        };
        eventSource.onerror = (err) => {
          console.error('EventSource failed:', err);
          eventSource.close();
        };
        await cacheEntryRemoved;
        eventSource.close();
      },
    }),
  }),
  overrideExisting: true,
});
export const { useHealthCheckQuery } = generatedApi; // Re-export the hooks you want to keep
export const { usePostQuery } = aggregatorApi; 
