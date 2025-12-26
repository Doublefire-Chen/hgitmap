import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { AuthProvider } from './context/AuthContext';
import { ThemeProvider } from './context/ThemeContext';
import { ToastProvider } from './context/ToastContext';
import ApiClientSetup from './components/ApiClientSetup';
import ProtectedRoute from './components/ProtectedRoute';
import SettingsLayout from './components/SettingsLayout';
import UserSettings from './components/UserSettings';
import Login from './pages/Login';
import Register from './pages/Register';
import Home from './pages/Home';
import PlatformManagement from './pages/PlatformManagement';
import OAuthCallback from './pages/OAuthCallback';
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
                  <SettingsLayout />
                </ProtectedRoute>
              }
            >
              <Route index element={<UserSettings />} />
              <Route path="platforms" element={<PlatformManagement />} />
              <Route path="themes" element={<HeatmapThemes />} />
              <Route path="themes/new" element={<ThemeEditor />} />
              <Route path="themes/:slug/edit" element={<ThemeEditor />} />
              <Route path="generation" element={<GenerationSettings />} />
              <Route path="sync" element={<SyncSettings />} />
            </Route>
            {/* Legacy routes - redirect to new structure */}
            <Route path="/platforms" element={<Navigate to="/settings/platforms" replace />} />
            <Route path="/admin/oauth-apps" element={<Navigate to="/settings/platforms" replace />} />
            <Route path="/heatmap/themes" element={<Navigate to="/settings/themes" replace />} />
            <Route path="/heatmap/themes/new" element={<Navigate to="/settings/themes/new" replace />} />
            <Route path="/heatmap/themes/:slug/edit" element={<Navigate to="/settings/themes/:slug/edit" replace />} />
            <Route path="/heatmap/settings" element={<Navigate to="/settings/generation" replace />} />
            <Route path="/sync/settings" element={<Navigate to="/settings/sync" replace />} />
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
