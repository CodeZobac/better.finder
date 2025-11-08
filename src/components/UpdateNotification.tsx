import { useUpdater } from '../hooks/useUpdater';
import { Download, X, CheckCircle, AlertCircle } from 'lucide-react';

export function UpdateNotification() {
  const { updateAvailable, updateVersion, updateInstalled, updateError, dismissUpdate } = useUpdater();

  if (!updateAvailable && !updateInstalled && !updateError) {
    return null;
  }

  return (
    <div className="fixed top-4 right-4 z-50 max-w-sm">
      {updateAvailable && (
        <div className="bg-blue-500 text-white rounded-lg shadow-lg p-4 flex items-start gap-3 animate-slideIn">
          <Download className="w-5 h-5 mt-0.5 flex-shrink-0" />
          <div className="flex-1">
            <h3 className="font-semibold mb-1">Update Available</h3>
            <p className="text-sm opacity-90">
              Version {updateVersion} is available. The update is being downloaded and will be installed automatically.
            </p>
          </div>
          <button
            onClick={dismissUpdate}
            className="text-white/80 hover:text-white transition-colors"
            aria-label="Dismiss"
          >
            <X className="w-5 h-5" />
          </button>
        </div>
      )}

      {updateInstalled && (
        <div className="bg-green-500 text-white rounded-lg shadow-lg p-4 flex items-start gap-3 animate-slideIn">
          <CheckCircle className="w-5 h-5 mt-0.5 flex-shrink-0" />
          <div className="flex-1">
            <h3 className="font-semibold mb-1">Update Installed</h3>
            <p className="text-sm opacity-90">
              The update has been installed successfully. Please restart the application to apply the changes.
            </p>
          </div>
          <button
            onClick={dismissUpdate}
            className="text-white/80 hover:text-white transition-colors"
            aria-label="Dismiss"
          >
            <X className="w-5 h-5" />
          </button>
        </div>
      )}

      {updateError && (
        <div className="bg-red-500 text-white rounded-lg shadow-lg p-4 flex items-start gap-3 animate-slideIn">
          <AlertCircle className="w-5 h-5 mt-0.5 flex-shrink-0" />
          <div className="flex-1">
            <h3 className="font-semibold mb-1">Update Failed</h3>
            <p className="text-sm opacity-90">
              Failed to download or install the update. Please try again later.
            </p>
          </div>
          <button
            onClick={dismissUpdate}
            className="text-white/80 hover:text-white transition-colors"
            aria-label="Dismiss"
          >
            <X className="w-5 h-5" />
          </button>
        </div>
      )}
    </div>
  );
}
