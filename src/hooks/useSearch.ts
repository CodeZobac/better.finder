import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { SearchResult } from '../types';

interface UseSearchResult {
  query: string;
  setQuery: (query: string) => void;
  results: SearchResult[];
  isLoading: boolean;
  error: string | null;
  executeResult: (result: SearchResult) => Promise<void>;
}

const DEBOUNCE_DELAY = 150; // milliseconds

/**
 * Custom hook for managing search functionality
 * Handles query state, debouncing, and communication with Tauri backend
 */
export function useSearch(): UseSearchResult {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<SearchResult[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  
  // Use ref to track the latest query for debouncing
  const debounceTimerRef = useRef<NodeJS.Timeout | null>(null);
  const abortControllerRef = useRef<AbortController | null>(null);

  /**
   * Performs the actual search by calling the Tauri backend
   */
  const performSearch = useCallback(async (searchQuery: string) => {
    // Cancel any pending search
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
    }

    // Create new abort controller for this search
    abortControllerRef.current = new AbortController();

    if (searchQuery.trim() === '') {
      setResults([]);
      setIsLoading(false);
      setError(null);
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      const searchResults = await invoke<SearchResult[]>('search_query', {
        query: searchQuery,
      });

      // Only update if this search wasn't aborted
      if (!abortControllerRef.current.signal.aborted) {
        setResults(searchResults);
        setError(null);
      }
    } catch (err) {
      // Only update error if this search wasn't aborted
      if (!abortControllerRef.current.signal.aborted) {
        const errorMessage = err instanceof Error ? err.message : 'Search failed';
        console.error('Search error:', errorMessage);
        setError(errorMessage);
        setResults([]);
      }
    } finally {
      // Only update loading state if this search wasn't aborted
      if (!abortControllerRef.current.signal.aborted) {
        setIsLoading(false);
      }
    }
  }, []);

  /**
   * Effect to handle debounced search when query changes
   */
  useEffect(() => {
    // Clear existing timer
    if (debounceTimerRef.current) {
      clearTimeout(debounceTimerRef.current);
    }

    // Set new timer for debounced search
    debounceTimerRef.current = setTimeout(() => {
      performSearch(query);
    }, DEBOUNCE_DELAY);

    // Cleanup function
    return () => {
      if (debounceTimerRef.current) {
        clearTimeout(debounceTimerRef.current);
      }
    };
  }, [query, performSearch]);

  /**
   * Executes a search result action
   */
  const executeResult = useCallback(async (result: SearchResult) => {
    try {
      await invoke('execute_result', { result });
      console.log('Result executed successfully:', result.title);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to execute result';
      console.error('Execute error:', errorMessage);
      throw new Error(errorMessage);
    }
  }, []);

  return {
    query,
    setQuery,
    results,
    isLoading,
    error,
    executeResult,
  };
}
