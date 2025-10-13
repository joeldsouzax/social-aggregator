import { api } from '@/services';
import { configureStore } from '@reduxjs/toolkit';
import { useDispatch, TypedUseSelectorHook, useSelector } from 'react-redux';



const store = configureStore({
  reducer: {
    [api.reducerPath]: api.reducer,
  },
  middleware: (getDefaultMiddleware) =>
    getDefaultMiddleware().concat(api.middleware),
      
});

export type RootState = ReturnType<typeof store.getState>;
export const useTixlysSelector: TypedUseSelectorHook<RootState> = useSelector;
export type AggregatorDispatch = typeof store.dispatch;

export const useAggregatorDispatch: () => AggregatorDispatch = useDispatch;

export default store;
