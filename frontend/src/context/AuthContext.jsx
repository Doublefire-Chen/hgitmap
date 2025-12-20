import { createContext, useContext, useState } from 'react';
import apiClient from '../api/client';

const AuthContext = createContext(null);

// eslint-disable-next-line react-refresh/only-export-components
export const useAuth = () => {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
};

// Helper function to get initial user from localStorage
const getInitialUser = () => {
  const token = localStorage.getItem('token');
  const savedUser = localStorage.getItem('user');

  if (token && savedUser) {
    try {
      return JSON.parse(savedUser);
    } catch {
      localStorage.removeItem('token');
      localStorage.removeItem('user');
      return null;
    }
  }
  return null;
};

export const AuthProvider = ({ children }) => {
  const [user, setUser] = useState(getInitialUser);

  const login = async (username, password) => {
    const data = await apiClient.login(username, password);
    localStorage.setItem('token', data.token);
    localStorage.setItem('user', JSON.stringify({
      id: data.user_id,
      username: data.username,
      is_admin: data.is_admin,
    }));
    setUser({
      id: data.user_id,
      username: data.username,
      is_admin: data.is_admin,
    });
    return data;
  };

  const register = async (username, password) => {
    const data = await apiClient.register(username, password);
    localStorage.setItem('token', data.token);
    localStorage.setItem('user', JSON.stringify({
      id: data.user_id,
      username: data.username,
      is_admin: data.is_admin,
    }));
    setUser({
      id: data.user_id,
      username: data.username,
      is_admin: data.is_admin,
    });
    return data;
  };

  const logout = () => {
    localStorage.removeItem('token');
    localStorage.removeItem('user');
    setUser(null);
  };

  return (
    <AuthContext.Provider value={{ user, login, register, logout }}>
      {children}
    </AuthContext.Provider>
  );
};
