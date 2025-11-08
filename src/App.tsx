import { useState, useEffect } from 'react';
import SearchBar from './components/SearchBar';
import Settings from './components/Settings';
import ToastContainer from './components/ToastContainer';
import { UpdateNotification } from './components/UpdateNotification';
import { useWindowVisibility } from './hooks/useWindowVisibility';
import { useTheme } from './hooks/useTheme';
import { useToast } from './hooks/useToast';
import { listen } from '@tauri-apps/api/event';

// Create a global toast context
export let globalToast: ReturnType<typeof useToast> | null = null;

function App() {
  const { isVisible, hideWindow } = useWindowVisibility();
  const { refreshTheme } = useTheme();
  const [settingsOpen, setSettingsOpen] = useState(false);
  const toast = useToast();

  // Make toast available globally
  useEffect(() => {
    globalToast = toast;
  }, [toast]);

  useEffect(() => {
    // Listen for settings open event (can be triggered from system tray)
    const unlisten = listen('open-settings', () => {
      setSettingsOpen(true);
    });

    // Listen for error events from backend
    const unlistenError = listen<{ title: string; message?: string }>('error', (event) => {
      toast.error(event.payload.title, event.payload.message);
    });

    // Listen for success events from backend
    const unlistenSuccess = listen<{ title: string; message?: string }>('success', (event) => {
      toast.success(event.payload.title, event.payload.message);
    });

    // Listen for warning events from backend
    const unlistenWarning = listen<{ title: string; message?: string }>('warning', (event) => {
      toast.warning(event.payload.title, event.payload.message);
    });

    return () => {
      unlisten.then((fn) => fn());
      unlistenError.then((fn) => fn());
      unlistenSuccess.then((fn) => fn());
      unlistenWarning.then((fn) => fn());
    };
  }, [toast]);

  const handleSettingsClose = () => {
    setSettingsOpen(false);
    // Refresh theme when settings are closed in case it changed
    refreshTheme();
  };

  // Handle click outside to close window
  const handleBackdropClick = (e: React.MouseEvent) => {
    if (e.target === e.currentTarget && !settingsOpen) {
      hideWindow();
    }
  };

  return (
    <div 
      className="min-h-screen bg-background flex items-start justify-center pt-8 px-4"
      onClick={handleBackdropClick}
    >
      <SearchBar 
        isVisible={isVisible} 
        onClose={hideWindow}
        onOpenSettings={() => setSettingsOpen(true)}
      />
      <Settings 
        isOpen={settingsOpen} 
        onClose={handleSettingsClose} 
      />
      <ToastContainer toasts={toast.toasts} onClose={toast.closeToast} />
      <UpdateNotification />
    </div>
  );
}

export default App;
