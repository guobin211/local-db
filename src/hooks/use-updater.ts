import { useState, useCallback } from 'react';
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

export interface UpdateInfo {
  available: boolean;
  currentVersion: string;
  latestVersion?: string;
  body?: string;
  date?: string;
}

export const useUpdater = () => {
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [isChecking, setIsChecking] = useState(false);
  const [isInstalling, setIsInstalling] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const checkForUpdates = useCallback(async () => {
    try {
      setIsChecking(true);
      setError(null);

      const update = await check();

      if (update) {
        setUpdateInfo({
          available: true,
          currentVersion: update.currentVersion,
          latestVersion: update.version,
          body: update.body,
          date: update.date
        });
      } else {
        setUpdateInfo({
          available: false,
          currentVersion: '0.1.0'
        });
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to check for updates');
      console.error('Update check failed:', err);
    } finally {
      setIsChecking(false);
    }
  }, []);

  const installUpdateNow = useCallback(async () => {
    try {
      setIsInstalling(true);
      setError(null);

      const update = await check();
      if (update) {
        await update.downloadAndInstall();
        await relaunch();
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to install update');
      console.error('Update installation failed:', err);
    } finally {
      setIsInstalling(false);
    }
  }, []);

  return {
    updateInfo,
    isChecking,
    isInstalling,
    error,
    checkForUpdates,
    installUpdateNow
  };
};
