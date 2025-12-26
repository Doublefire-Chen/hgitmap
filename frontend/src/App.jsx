import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { AuthProvider } from './context/AuthContext';
import { ThemeProvider } from './context/ThemeContext';
import { ToastProvider } from './context/ToastContext';
import ApiClientSetup from './components/ApiClientSetup';
import ProtectedRoute from './components/ProtectedRoute';
import Login from './pages/Login';
import Register from './pages/Register';
import Home from './pages/Home';
import OAuthCallback from './pages/OAuthCallback';
import Settings from './pages/Settings';
import AdminOAuthApps from './pages/AdminOAuthApps';
import HeatmapThemes from './pages/HeatmapThemes';
import ThemeEditor from './pages/ThemeEditor';
import GenerationSettings from './pages/GenerationSettings';
import SyncSettings from './pages/SyncSettings';

function App() {
  return (
    <BrowserRouter>
      <ThemeProvider>
        <ToastProvider>
          <ApiClientSetup>
            <AuthProvider>
              <Routes>
            <Route path="/login" element={<Login />} />
            <Route path="/register" element={<Register />} />
            <Route
              path="/"
              element={
                <ProtectedRoute>
                  <Home />
                </ProtectedRoute>
              }
            />
            <Route
              path="/settings"
              element={
                <ProtectedRoute>
                  <Settings />
                </ProtectedRoute>
              }
            />
            <Route
              path="/admin/oauth-apps"
              element={
                <ProtectedRoute>
                  <AdminOAuthApps />
                </ProtectedRoute>
              }
            />
            <Route
              path="/heatmap/themes"
              element={
                <ProtectedRoute>
                  <HeatmapThemes />
                </ProtectedRoute>
              }
            />
            <Route
              path="/heatmap/themes/new"
              element={
                <ProtectedRoute>
                  <ThemeEditor />
                </ProtectedRoute>
              }
            />
            <Route
              path="/heatmap/themes/:slug/edit"
              element={
                <ProtectedRoute>
                  <ThemeEditor />
                </ProtectedRoute>
              }
            />
            <Route
              path="/heatmap/settings"
              element={
                <ProtectedRoute>
                  <GenerationSettings />
                </ProtectedRoute>
              }
            />
            <Route
              path="/sync/settings"
              element={
                <ProtectedRoute>
                  <SyncSettings />
                </ProtectedRoute>
              }
            />
            <Route
              path="/oauth/callback"
              element={
                <ProtectedRoute>
                  <OAuthCallback />
                </ProtectedRoute>
              }
            />
            <Route path="*" element={<Navigate to="/" replace />} />
          </Routes>
        </AuthProvider>
          </ApiClientSetup>
        </ToastProvider>
      </ThemeProvider>
    </BrowserRouter>
  );
}

export default App;
