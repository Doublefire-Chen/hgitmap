import { createContext, useContext, useState, useCallback } from 'react';
import Toast from '../components/Toast';
import '../components/Toast.css';

const ToastContext = createContext(null);

// eslint-disable-next-line react-refresh/only-export-components
export const useToast = () => {
  const context = useContext(ToastContext);
  if (!context) {
    throw new Error('useToast must be used within a ToastProvider');
  }
  return context;
};

export const ToastProvider = ({ children }) => {
  const [toasts, setToasts] = useState([]);

  const showToast = useCallback((message, type = 'error', duration = 5000, showProgress = false) => {
    const id = Date.now();
    setToasts((prev) => [...prev, { id, message, type, duration, showProgress }]);
  }, []);

  const hideToast = useCallback((id) => {
    setToasts((prev) => prev.filter((toast) => toast.id !== id));
  }, []);

  const error = useCallback((message, duration, showProgress) => {
    showToast(message, 'error', duration, showProgress);
  }, [showToast]);

  const success = useCallback((message, duration, showProgress) => {
    showToast(message, 'success', duration, showProgress);
  }, [showToast]);

  const warning = useCallback((message, duration, showProgress) => {
    showToast(message, 'warning', duration, showProgress);
  }, [showToast]);

  return (
    <ToastContext.Provider value={{ showToast, error, success, warning }}>
      {children}
      <div className="toast-container">
        {toasts.map((toast) => (
          <Toast
            key={toast.id}
            message={toast.message}
            type={toast.type}
            duration={toast.duration}
            showProgress={toast.showProgress}
            onClose={() => hideToast(toast.id)}
          />
        ))}
      </div>
    </ToastContext.Provider>
  );
};
