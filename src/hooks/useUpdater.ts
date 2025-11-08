import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

interface UpdaterState {
  updateAvailable: boolean;
  updateVersion: string | null;
  updateInstalled: boolean;
  updateError: string | null;
  isChecking: boolean;
}

export function useUpdater() {
  const [state, setState] = useState<UpdaterState>({
    updateAvailable: false,
    updateVersion: null,
    updateInstalled: false,
    updateError: null,
    isChecking: false,
  });

  useEffect(() => {
    // Listen for update-available event
    const unlistenAvailable = listen<string>('update-available', (event) => {
      console.log('Update available:', event.payload);
      setState((prev) => ({
        ...prev,
        updateAvailable: true,
        updateVersion: event.payload,
      }));
    });

    // Listen for update-installed event
    const unlistenInstalled = listen('update-installed', () => {
      console.log('Update installed');
      setState((prev) => ({
        ...prev,
        updateInstalled: true,
        updateAvailable: false,
      }));
    });

    // Listen for update-error event
    const unlistenError = listen<string>('update-error', (event) => {
      console.error('Update error:', event.payload);
      setState((prev) => ({
        ...prev,
        updateError: event.payload,
        updateAvailable: false,
      }));
    });

    // Cleanup listeners
    return () => {
      unlistenAvailable.then((fn) => fn());
      unlistenInstalled.then((fn) => fn());
      unlistenError.then((fn) => fn());
    };
  }, []);

  const checkForUpdates = async () => {
    setState((prev) => ({ ...prev, isChecking: true, updateError: null }));
    try {
      const result = await invoke<string>('check_for_updates_manual');
      console.log('Update check result:', result);
      setState((prev) => ({ ...prev, isChecking: false }));
      return result;
    } catch (error) {
      console.error('Failed to check for updates:', error);
      setState((prev) => ({
        ...prev,
        isChecking: false,
        updateError: String(error),
      }));
      throw error;
    }
  };

  const dismissUpdate = () => {
    setState((prev) => ({
      ...prev,
      updateAvailable: false,
      updateVersion: null,
      updateInstalled: false,
      updateError: null,
    }));
  };

  return {
    ...state,
    checkForUpdates,
    dismissUpdate,
  };
}
