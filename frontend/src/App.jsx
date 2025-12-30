import { BrowserRouter, Routes, Route, Navigate, useParams } from 'react-router-dom';
import { useEffect } from 'react';
import { AuthProvider } from './context/AuthContext';
import { ThemeProvider, useTheme } from './context/ThemeContext';
import { ToastProvider } from './context/ToastContext';
import ApiClientSetup from './components/ApiClientSetup';
import ProtectedRoute from './components/ProtectedRoute';
import SettingsLayout from './components/SettingsLayout';
import UserSettings from './components/UserSettings';
import Login from './pages/Login';
import Register from './pages/Register';
import Landing from './pages/Landing';
import Profile from './pages/Profile';
import PlatformManagement from './pages/PlatformManagement';
import OAuthCallback from './pages/OAuthCallback';
import HeatmapThemes from './pages/HeatmapThemes';
import ThemeEditor from './pages/ThemeEditor';
import GenerationSettings from './pages/GenerationSettings';
import SyncSettings from './pages/SyncSettings';
import Footer from './components/Footer';
import { updateFavicon } from './utils/favicon';

// Helper component for redirecting with parameters
function ThemeEditRedirect() {
  const { slug } = useParams();
  return <Navigate to={`/settings/themes/${slug}/edit`} replace />;
}

// Helper component for redirecting to user profile
function RootRedirect() {
  const user = JSON.parse(localStorage.getItem('user') || 'null');
  if (user && user.username) {
    return <Navigate to={`/${user.username}`} replace />;
  }
  return <Landing />;
}

// Component to handle dynamic favicon
function FaviconUpdater() {
  const { theme } = useTheme();

  useEffect(() => {
    // Update favicon when theme changes or component mounts (page refresh)
    updateFavicon(theme);
  }, [theme]);

  return null;
}

function App() {
  return (
    <BrowserRouter>
      <ThemeProvider>
        <FaviconUpdater />
        <ToastProvider>
          <ApiClientSetup>
            <AuthProvider>
              <div style={{ minHeight: '100vh', display: 'flex', flexDirection: 'column' }}>
                <div style={{ flex: 1 }}>
                  <Routes>
                    <Route path="/login" element={<Login />} />
                    <Route path="/register" element={<Register />} />
                    <Route path="/" element={<RootRedirect />} />
                    {/* Public profile route */}
                    <Route path="/:username" element={<Profile />} />
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
                    <Route path="/heatmap/themes/:slug/edit" element={<ThemeEditRedirect />} />
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
                </div>
                <Footer />
              </div>
            </AuthProvider>
          </ApiClientSetup>
        </ToastProvider>
      </ThemeProvider>
    </BrowserRouter>
  );
}

export default App;
