import { createApi, fetchBaseQuery } from '@reduxjs/toolkit/query/react';

const baseQuery = fetchBaseQuery({
  baseUrl: import.meta.env.VITE_BASE_URL,
  credentials: 'same-origin',
  mode: 'cors',
});

export const api = createApi({
  baseQuery,
  refetchOnReconnect: true,
  tagTypes: [],
  // enpoints are injected later
  endpoints: () => ({}),
});
