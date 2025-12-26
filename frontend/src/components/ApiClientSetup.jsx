import { useEffect } from 'react';
import { useToast } from '../context/ToastContext';
import apiClient from '../api/client';

const ApiClientSetup = ({ children }) => {
  const { error } = useToast();

  useEffect(() => {
    // Set up the session expired callback
    apiClient.setSessionExpiredCallback(() => {
      error('Your session has expired. Please log in again.', 3000, true); // true = show progress bar
    });
  }, [error]);

  return children;
};

export default ApiClientSetup;
