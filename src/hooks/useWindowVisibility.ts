import { useState, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

export const useWindowVisibility = () => {
  const [isVisible, setIsVisible] = useState(true);

  useEffect(() => {
    // Listen for hotkey press events
    const unlistenPromise = listen('hotkey-pressed', () => {
      setIsVisible((prev) => !prev);
    });

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  // Effect to show/hide window based on visibility state
  useEffect(() => {
    if (isVisible) {
      invoke('show_window').catch((error) => {
        console.error('Failed to show window:', error);
      });
    } else {
      invoke('hide_window').catch((error) => {
        console.error('Failed to hide window:', error);
      });
    }
  }, [isVisible]);

  const showWindow = () => setIsVisible(true);
  const hideWindow = () => setIsVisible(false);
  const toggleWindow = () => setIsVisible((prev) => !prev);

  return {
    isVisible,
    showWindow,
    hideWindow,
    toggleWindow,
  };
};
