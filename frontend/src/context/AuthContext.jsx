import { createContext, useContext, useState, useEffect } from 'react';
import apiClient from '../api/client';

const AuthContext = createContext(null);

export const useAuth = () => {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
};

export const AuthProvider = ({ children }) => {
  const [user, setUser] = useState(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    // Check if user is logged in on mount
    const token = localStorage.getItem('token');
    const savedUser = localStorage.getItem('user');

    if (token && savedUser) {
      try {
        setUser(JSON.parse(savedUser));
      } catch (e) {
        localStorage.removeItem('token');
        localStorage.removeItem('user');
      }
    }

    setLoading(false);
  }, []);

  const login = async (username, password) => {
    const data = await apiClient.login(username, password);
    localStorage.setItem('token', data.token);
    localStorage.setItem('user', JSON.stringify({
      id: data.user_id,
      username: data.username,
    }));
    setUser({
      id: data.user_id,
      username: data.username,
    });
    return data;
  };

  const register = async (username, password, email) => {
    const data = await apiClient.register(username, password, email);
    localStorage.setItem('token', data.token);
    localStorage.setItem('user', JSON.stringify({
      id: data.user_id,
      username: data.username,
    }));
    setUser({
      id: data.user_id,
      username: data.username,
    });
    return data;
  };

  const logout = () => {
    localStorage.removeItem('token');
    localStorage.removeItem('user');
    setUser(null);
  };

  return (
    <AuthContext.Provider value={{ user, login, register, logout, loading }}>
      {children}
    </AuthContext.Provider>
  );
};
